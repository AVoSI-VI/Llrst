//! Warnings over a lowered body. They do not fail the build.
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
    matches!(s, Stmt::Drop(_) | Stmt::StorageDead(_) | Stmt::Nop | Stmt::InlineLoans { .. })
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
        Rvalue::Use(op) | Rvalue::Neg(op) | Rvalue::WNeg(op) | Rvalue::Not(op) | Rvalue::Cast(op, _) => mark_op(read, op),
        Rvalue::Ref { place, .. } | Rvalue::Discr(place) => mark_place(read, place),
        Rvalue::Bin(_, a, b) | Rvalue::PtrAdd(a, b, _) | Rvalue::PtrEq(a, b) => {
            mark_op(read, a);
            mark_op(read, b);
        }
        Rvalue::Struct(_, ops) | Rvalue::Variant(_, _, ops) => ops.iter().for_each(|o| mark_op(read, o)),
        Rvalue::PtrNull | Rvalue::Argc | Rvalue::Argv | Rvalue::Sizeof(_) => {}
    }
}

fn mark_term(read: &mut [bool], t: &Term) {
    match t {
        Term::If(op, ..) | Term::Switch(op, ..) => mark_op(read, op),
        Term::Call { func, args, .. } => {
            if let Func::Ptr(op) = func {
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
        Lit(_) | Break | Continue | Sizeof(_) | Name(_) => None,
    };
}
