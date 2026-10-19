//! Closure checking. A closure copies or moves whole bindings of the enclosing function.
use super::Checker;
use crate::ast::{self, ExprKind as A};
use crate::diag::{Diag, PResult, Span};
use crate::thir::*;
use crate::ty::Ty;
use std::collections::HashSet;

fn mk(kind: ExprKind, ty: Ty, span: Span) -> Expr {
    Expr { kind, ty, span }
}

impl<'a> Checker<'a> {
    pub fn check_clos(&mut self, params: &[ast::Param], ret: &ast::TyExpr, body: &[ast::Expr], sp: Span) -> PResult<Expr> {
        if self.in_clos {
            return Err(Diag::error(sp, "clos.nested"));
        }
        let ret = self.resolve_ty(ret)?;
        let mut bound: HashSet<String> = params.iter().map(|p| p.name.clone()).collect();
        let found = free_caps(self, &mut bound, body);
        let start = self.locals.len() as u32;
        self.push_scope();
        let mut cap_ids = Vec::new();
        let mut cap_exprs = Vec::new();
        let mut cap_tys = Vec::new();
        for (name, outer) in found {
            let ty = self.locals[outer.0 as usize].ty.clone();
            let id = self.declare(&name, ty.clone(), false, sp)?;
            cap_ids.push(id);
            cap_tys.push(ty.clone());
            cap_exprs.push(mk(ExprKind::Local(outer), ty, sp));
        }
        let mut user_ids = Vec::new();
        let mut user_tys = Vec::new();
        for p in params {
            let ty = self.resolve_ty(&p.ty)?;
            user_ids.push(self.declare(&p.name, ty.clone(), false, p.span)?);
            user_tys.push(ty);
        }
        let saved = self.in_clos;
        self.in_clos = true;
        let checked = self.check_body(body, &ret, sp);
        self.in_clos = saved;
        self.pop_scope();
        let mut checked = checked?;
        remap_expr(&mut checked, start);
        let locals = self.locals[start as usize..].to_vec();
        for l in &mut self.locals[start as usize..] {
            l.name.insert(0, '_');
        }
        let params: Vec<LocalId> = cap_ids.into_iter().chain(user_ids).map(|id| LocalId(id.0 - start)).collect();
        let inner = ClosBody { locals, params, ret: ret.clone(), body: Box::new(checked), caps: cap_tys, user: user_tys.clone() };
        let ty = Ty::Clos(user_tys, Box::new(ret));
        Ok(mk(ExprKind::Closure { caps: cap_exprs, inner }, ty, sp))
    }
}

fn free_caps(ck: &Checker, bound: &mut HashSet<String>, body: &[ast::Expr]) -> Vec<(String, LocalId)> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    walk_seq(ck, bound, body, &mut out, &mut seen);
    out
}

fn walk_seq(ck: &Checker, bound: &mut HashSet<String>, xs: &[ast::Expr], out: &mut Vec<(String, LocalId)>, seen: &mut HashSet<String>) {
    for e in xs {
        walk(ck, bound, e, out, seen);
    }
}

fn note(ck: &Checker, bound: &HashSet<String>, name: &str, out: &mut Vec<(String, LocalId)>, seen: &mut HashSet<String>) {
    if bound.contains(name) || !seen.insert(name.to_string()) {
        return;
    }
    if let Some(id) = ck.lookup_local(name) {
        out.push((name.to_string(), id));
    }
}

fn walk(ck: &Checker, bound: &mut HashSet<String>, e: &ast::Expr, out: &mut Vec<(String, LocalId)>, seen: &mut HashSet<String>) {
    match &e.kind {
        A::Name(n) => note(ck, bound, n, out, seen),
        A::Call { callee, args, .. } => {
            note(ck, bound, callee, out, seen);
            walk_seq(ck, bound, args, out, seen);
        }
        A::Let { name, init, .. } => {
            if let Some(init) = init {
                walk(ck, bound, init, out, seen);
            }
            bound.insert(name.clone());
        }
        A::For { name, lo, hi, body } => {
            walk(ck, bound, lo, out, seen);
            walk(ck, bound, hi, out, seen);
            let mut inner = bound.clone();
            inner.insert(name.clone());
            walk_seq(ck, &mut inner, body, out, seen);
        }
        A::Match { scrut, arms } => {
            walk(ck, bound, scrut, out, seen);
            for a in arms {
                let mut inner = bound.clone();
                bind_pat(&mut inner, &a.pat);
                walk(ck, &mut inner, &a.body, out, seen);
            }
        }
        A::Clos { params, body, .. } => {
            let mut inner = bound.clone();
            for p in params {
                inner.insert(p.name.clone());
            }
            walk_seq(ck, &mut inner, body, out, seen);
        }
        A::Set { place, value } => {
            walk(ck, bound, place, out, seen);
            walk(ck, bound, value, out, seen);
        }
        A::Do(xs) | A::Loop(xs) => walk_seq(ck, bound, xs, out, seen),
        A::If { cond, then, els } => {
            walk(ck, bound, cond, out, seen);
            walk(ck, bound, then, out, seen);
            if let Some(els) = els {
                walk(ck, bound, els, out, seen);
            }
        }
        A::While { cond, body } => {
            walk(ck, bound, cond, out, seen);
            walk_seq(ck, bound, body, out, seen);
        }
        A::Return(x) => {
            if let Some(x) = x {
                walk(ck, bound, x, out, seen);
            }
        }
        A::Ref { place, .. } | A::Deref(place) | A::Not(place) | A::Neg(place) | A::WNeg(place) | A::Field { base: place, .. } | A::Cast { expr: place, .. }
        | A::Try(place) | A::Panic(place) | A::Assert(place) | A::Drop(place) => walk(ck, bound, place, out, seen),
        A::Bin { l, r, .. } | A::And(l, r) | A::Or(l, r) | A::AssertEq(l, r) => {
            walk(ck, bound, l, out, seen);
            walk(ck, bound, r, out, seen);
        }
        A::CallPtr { callee, args } => {
            walk(ck, bound, callee, out, seen);
            walk_seq(ck, bound, args, out, seen);
        }
        A::Lit(_) | A::Break | A::Continue | A::Sizeof(_) => {}
    }
}

fn bind_pat(bound: &mut HashSet<String>, p: &ast::Pat) {
    match p {
        ast::Pat::Bind(n, _) => {
            bound.insert(n.clone());
        }
        ast::Pat::Variant { subs, .. } => subs.iter().for_each(|s| bind_pat(bound, s)),
        ast::Pat::Wild(_) | ast::Pat::Lit(_, _) => {}
    }
}

fn remap_expr(e: &mut Expr, start: u32) {
    let sub = |l: &mut LocalId| l.0 -= start;
    match &mut e.kind {
        ExprKind::Local(l) => sub(l),
        ExprKind::Let { local, init } => {
            sub(local);
            if let Some(x) = init {
                remap_expr(x, start);
            }
        }
        ExprKind::For { local, lo, hi, body } => {
            sub(local);
            remap_expr(lo, start);
            remap_expr(hi, start);
            remap_expr(body, start);
        }
        ExprKind::Assign { place, value } => {
            remap_expr(place, start);
            remap_expr(value, start);
        }
        ExprKind::Block(xs) => xs.iter_mut().for_each(|x| remap_expr(x, start)),
        ExprKind::If { cond, then, els } => {
            remap_expr(cond, start);
            remap_expr(then, start);
            if let Some(x) = els {
                remap_expr(x, start);
            }
        }
        ExprKind::While { cond, body } => {
            remap_expr(cond, start);
            remap_expr(body, start);
        }
        ExprKind::Loop(b) => remap_expr(b, start),
        ExprKind::Return(v) => {
            if let Some(x) = v {
                remap_expr(x, start);
            }
        }
        ExprKind::Match { scrut, arms, .. } => {
            remap_expr(scrut, start);
            for a in arms {
                remap_pat(&mut a.pat, start);
                remap_expr(&mut a.body, start);
            }
        }
        ExprKind::Ref { place, .. } | ExprKind::Deref(place) | ExprKind::Field { base: place, .. } | ExprKind::Cast(place) | ExprKind::Neg(place)
        | ExprKind::WNeg(place) | ExprKind::Not(place) => remap_expr(place, start),
        ExprKind::Bin { l, r, .. } | ExprKind::And(l, r) | ExprKind::Or(l, r) => {
            remap_expr(l, start);
            remap_expr(r, start);
        }
        ExprKind::Call { callee, args } => {
            if let Callee::Ptr(x) | Callee::Clos(x) = callee {
                remap_expr(x, start);
            }
            args.iter_mut().for_each(|x| remap_expr(x, start));
        }
        ExprKind::Struct { fields, .. } | ExprKind::Variant { fields, .. } => fields.iter_mut().for_each(|x| remap_expr(x, start)),
        ExprKind::Try { expr, .. } => remap_expr(expr, start),
        ExprKind::DynPack { data, .. } => remap_expr(data, start),
        ExprKind::Closure { .. } => {}
        ExprKind::Lit(_) | ExprKind::Const(_) | ExprKind::Break | ExprKind::Continue | ExprKind::FnRef(_) => {}
    }
}

fn remap_pat(p: &mut Pat, start: u32) {
    match p {
        Pat::Bind(l) => l.0 -= start,
        Pat::Variant { subs, .. } => subs.iter_mut().for_each(|s| remap_pat(s, start)),
        Pat::Wild | Pat::Lit(_) => {}
    }
}
