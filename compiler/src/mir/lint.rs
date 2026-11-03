//! Warnings over a lowered body. A warning fails the build only when `--deny` names its code.
use super::*;
use crate::diag::{Diag, Diags};

pub fn lint(diags: &Diags, body: &Body) {
    warn_unreachable(diags, body);
    warn_unused(diags, body);
}

pub fn lint_unused(prog: &crate::resolve::Program, diags: &Diags) {
    note_bodies(prog);
    let uses = prog.used_uses.borrow();
    let fns = prog.used_fns.borrow();
    for (mi, m) in prog.modules.iter().enumerate() {
        if m.is_std {
            continue;
        }
        for (alias, span) in &m.use_span {
            if !uses.contains(&(mi as u32, alias.clone())) {
                diags.push(Diag::warn(*span, "use.unused").kv("name", alias.clone()));
            }
        }
    }
    for (i, d) in prog.defs.iter().enumerate() {
        if prog.modules[d.module as usize].is_std {
            continue;
        }
        let crate::resolve::DefKind::Fn(f) = &d.kind else { continue };
        if d.is_pub || f.is_extern || fns.contains(&crate::ty::DefId(i as u32)) {
            continue;
        }
        if d.name == "main" && d.module == prog.main_module {
            continue;
        }
        diags.push(Diag::warn(d.span, "fn.unused").kv("name", d.name.clone()));
    }
}

pub fn lint_generics(prog: &crate::resolve::Program, diags: &Diags, instantiated: &std::collections::HashSet<crate::ty::DefId>) {
    for (i, d) in prog.defs.iter().enumerate() {
        if prog.modules[d.module as usize].is_std {
            continue;
        }
        let id = crate::ty::DefId(i as u32);
        let crate::resolve::DefKind::Fn(f) = &d.kind else { continue };
        if f.is_extern || f.generics.is_empty() || instantiated.contains(&id) {
            continue;
        }
        lint_generic(diags, f, d.span);
    }
}

struct Bind {
    name: String,
    span: crate::diag::Span,
    used: bool,
}

fn lint_generic(diags: &Diags, f: &crate::resolve::FnDef, span: crate::diag::Span) {
    let mut scope: Vec<Bind> = f.params.iter().filter(|(n, _)| n != "_").map(|(n, _)| Bind { name: n.clone(), span, used: false }).collect();
    for e in &f.body {
        walk_expr(diags, e, &mut scope);
    }
    warn_binds(diags, &mut scope, 0);
}

fn warn_binds(diags: &Diags, scope: &mut Vec<Bind>, keep: usize) {
    while scope.len() > keep {
        let b = scope.pop().unwrap();
        if !b.used && !b.name.starts_with('_') {
            diags.push(Diag::warn(b.span, "name.unused").kv("name", b.name));
        }
    }
}

fn mark_bind(scope: &mut [Bind], name: &str) {
    if let Some(b) = scope.iter_mut().rev().find(|b| b.name == name) {
        b.used = true;
    }
}

fn walk_seq(diags: &Diags, xs: &[crate::ast::Expr], scope: &mut Vec<Bind>) {
    let n = scope.len();
    for e in xs {
        walk_expr(diags, e, scope);
    }
    warn_binds(diags, scope, n);
}

fn walk_expr(diags: &Diags, e: &crate::ast::Expr, scope: &mut Vec<Bind>) {
    use crate::ast::ExprKind::*;
    match &e.kind {
        Name(n) => mark_bind(scope, n),
        Call { callee, .. } => mark_bind(scope, callee),
        Let { name, init, .. } => {
            if let Some(init) = init {
                walk_expr(diags, init, scope);
            }
            if name != "_" {
                scope.push(Bind { name: name.clone(), span: e.span, used: false });
            }
            return;
        }
        _ => {}
    }
    match &e.kind {
        Let { .. } | Name(_) | Lit(_) | Break | Continue | Sizeof(_) => {}
        Set { place, value } => {
            walk_expr(diags, place, scope);
            walk_expr(diags, value, scope);
        }
        Do(xs) | Loop(xs) => walk_seq(diags, xs, scope),
        If { cond, then, els } => {
            walk_expr(diags, cond, scope);
            walk_expr(diags, then, scope);
            if let Some(els) = els {
                walk_expr(diags, els, scope);
            }
        }
        While { cond, body } => {
            walk_expr(diags, cond, scope);
            walk_seq(diags, body, scope);
        }
        For { lo, hi, body, .. } => {
            walk_expr(diags, lo, scope);
            walk_expr(diags, hi, scope);
            walk_seq(diags, body, scope);
        }
        Return(x) => {
            if let Some(x) = x {
                walk_expr(diags, x, scope);
            }
        }
        Match { scrut, arms } => {
            walk_expr(diags, scrut, scope);
            for a in arms {
                walk_expr(diags, &a.body, scope);
            }
        }
        Ref { place, .. } | Deref(place) | Not(place) | Neg(place) | WNeg(place) | Field { base: place, .. } | Cast { expr: place, .. } | Try(place)
        | Panic(place) | Assert(place) | Drop(place) => walk_expr(diags, place, scope),
        Bin { l, r, .. } | And(l, r) | Or(l, r) | AssertEq(l, r) => {
            walk_expr(diags, l, scope);
            walk_expr(diags, r, scope);
        }
        Call { args, .. } => args.iter().for_each(|x| walk_expr(diags, x, scope)),
        CallPtr { callee, args } => {
            walk_expr(diags, callee, scope);
            args.iter().for_each(|x| walk_expr(diags, x, scope));
        }
        Clos { params, body, .. } => {
            let n = scope.len();
            for p in params {
                scope.push(Bind { name: p.name.clone(), span: p.span, used: false });
            }
            walk_seq(diags, body, scope);
            scope.truncate(n);
        }
    }
}

fn warn_unreachable(diags: &Diags, body: &Body) {
    let n = body.blocks.len();
    let mut seen = vec![false; n];
    let mut stack = vec![0u32];
    if n > 0 {
        seen[0] = true;
    }
    while let Some(b) = stack.pop() {
        for s in body.successors(b) {
            if !seen[s as usize] {
                seen[s as usize] = true;
                stack.push(s);
            }
        }
    }
    for (i, reach) in seen.iter().enumerate() {
        if *reach {
            continue;
        }
        let blk = &body.blocks[i];
        let interesting = blk.stmts.iter().find(|(s, _)| !is_cleanup(s));
        if let Some((_, span)) = interesting {
            diags.push(Diag::warn(*span, "flow.unreachable"));
        }
    }
}

fn is_cleanup(s: &Stmt) -> bool {
    matches!(s, Stmt::Drop(_) | Stmt::StorageDead(_) | Stmt::Nop | Stmt::InlineLoans { .. } | Stmt::TracePush { .. } | Stmt::TracePop { .. })
}

fn warn_unused(diags: &Diags, body: &Body) {
    let mut read = vec![false; body.locals.len()];
    for blk in &body.blocks {
        for (s, _) in &blk.stmts {
            mark_stmt(&mut read, s);
        }
        mark_term(&mut read, &blk.term);
    }
    for (i, loc) in body.locals.iter().enumerate().skip(1) {
        if loc.name.starts_with('_') || read[i] {
            continue;
        }
        diags.push(Diag::warn(loc.span, "name.unused").kv("name", loc.name.clone()));
    }
}

fn mark_place(read: &mut [bool], p: &Place) {
    read[p.local as usize] = true;
}

fn mark_op(read: &mut [bool], op: &Operand) {
    if let Some(p) = op.place() {
        mark_place(read, p);
    }
}

fn mark_stmt(read: &mut [bool], s: &Stmt) {
    let Stmt::Assign(_, rv) = s else { return };
    match rv {
        Rvalue::Use(op) | Rvalue::Neg(op) | Rvalue::WNeg(op) | Rvalue::Not(op) | Rvalue::Cast(op, _) | Rvalue::BoxPack { value: op, .. } => {
            mark_op(read, op);
        }
        Rvalue::Ref { place, .. } | Rvalue::BoxBorrow { place, .. } | Rvalue::Discr(place) => mark_place(read, place),
        Rvalue::Bin(_, a, b) | Rvalue::PtrAdd(a, b, _) | Rvalue::PtrEq(a, b) => {
            mark_op(read, a);
            mark_op(read, b);
        }
        Rvalue::Struct(_, ops) | Rvalue::Variant(_, _, ops) | Rvalue::Closure { caps: ops, .. } => ops.iter().for_each(|o| mark_op(read, o)),
        Rvalue::PtrNull | Rvalue::Argc | Rvalue::Argv | Rvalue::Sizeof(_) | Rvalue::Vtable(..) | Rvalue::Static(_) => {}
    }
}

fn mark_term(read: &mut [bool], t: &Term) {
    match t {
        Term::If(op, ..) | Term::Switch(op, ..) => mark_op(read, op),
        Term::Call { func, args, .. } => {
            if let Func::Ptr(op) | Func::Clos(op) = func {
                mark_op(read, op);
            }
            args.iter().for_each(|a| mark_op(read, a));
        }
        Term::Return => read[0] = true,
        Term::Goto(_) | Term::Unreachable | Term::None => {}
    }
}

fn note_bodies(prog: &crate::resolve::Program) {
    for d in &prog.defs {
        let crate::resolve::DefKind::Fn(f) = &d.kind else { continue };
        for e in &f.body {
            note_expr(prog, d.module, e);
        }
    }
}

fn note_alias(prog: &crate::resolve::Program, module: u32, name: &str) {
    let Some((alias, _)) = name.split_once('.') else { return };
    if prog.modules[module as usize].use_span.contains_key(alias) {
        prog.used_uses.borrow_mut().insert((module, alias.to_string()));
    }
}

fn note_expr(prog: &crate::resolve::Program, module: u32, e: &crate::ast::Expr) {
    use crate::ast::ExprKind::*;
    let one = |x: &crate::ast::Expr| note_expr(prog, module, x);
    let many = |xs: &[crate::ast::Expr]| xs.iter().for_each(|x| note_expr(prog, module, x));
    match &e.kind {
        Name(n) | Call { callee: n, .. } => note_alias(prog, module, n),
        _ => {}
    }
    match &e.kind {
        Let { init, .. } => init.as_deref().map(one),
        Set { place, value } => {
            one(place);
            Some(one(value))
        }
        Do(xs) | Loop(xs) => {
            many(xs);
            None
        }
        If { cond, then, els } => {
            one(cond);
            one(then);
            els.as_deref().map(one)
        }
        While { cond, body } => {
            one(cond);
            many(body);
            None
        }
        For { lo, hi, body, .. } => {
            one(lo);
            one(hi);
            many(body);
            None
        }
        Return(x) => x.as_deref().map(one),
        Match { scrut, arms } => {
            one(scrut);
            arms.iter().for_each(|a| one(&a.body));
            None
        }
        Ref { place, .. } | Deref(place) | Not(place) | Neg(place) | WNeg(place) | Field { base: place, .. } | Cast { expr: place, .. } | Try(place)
        | Panic(place) | Assert(place) | Drop(place) => Some(one(place)),
        Bin { l, r, .. } | And(l, r) | Or(l, r) | AssertEq(l, r) => {
            one(l);
            Some(one(r))
        }
        Call { args, .. } => {
            many(args);
            None
        }
        CallPtr { callee, args } => {
            one(callee);
            many(args);
            None
        }
        Clos { body, .. } => {
            many(body);
            None
        }
        Lit(_) | Break | Continue | Sizeof(_) | Name(_) => None,
    };
}
