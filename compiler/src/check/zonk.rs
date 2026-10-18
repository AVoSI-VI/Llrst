//! Replaces inference variables with their solutions throughout a body.
use super::Checker;
use crate::diag::{Diag, PResult};
use crate::thir::*;
use crate::ty::Ty;

pub fn zonk_body(c: &mut Checker, body: Expr) -> PResult<Expr> {
    for i in 0..c.locals.len() {
        let t = c.zonk_ty(&c.locals[i].ty);
        if t.has_infer() {
            return Err(Diag::error(c.locals[i].span, "type.infer").kv("name", c.locals[i].name.clone()).kv("got", c.show(&t)));
        }
        c.locals[i].ty = t;
    }
    zonk_expr(c, body)
}

fn zonk_tys(c: &Checker, tys: &[Ty], span: crate::diag::Span) -> PResult<Vec<Ty>> {
    tys.iter()
        .map(|t| {
            let z = c.zonk_ty(t);
            if z.has_infer() {
                Err(Diag::error(span, "type.infer").kv("got", c.show(&z)).kv("fix", "add <T> to the call"))
            } else {
                Ok(z)
            }
        })
        .collect()
}

fn zonk_expr(c: &mut Checker, e: Expr) -> PResult<Expr> {
    let ty = c.zonk_ty(&e.ty);
    if ty.has_infer() {
        return Err(Diag::error(e.span, "type.infer").kv("got", c.show(&ty)));
    }
    let span = e.span;
    let bx = |c: &mut Checker, x: Box<Expr>| -> PResult<Box<Expr>> { Ok(Box::new(zonk_expr(c, *x)?)) };
    let kind = match e.kind {
        ExprKind::Lit(_) | ExprKind::Local(_) | ExprKind::Const(_) | ExprKind::Break | ExprKind::Continue | ExprKind::FnRef(_) => e.kind,
        ExprKind::Let { local, init } => ExprKind::Let {
            local,
            init: match init {
                Some(x) => Some(bx(c, x)?),
                None => None,
            },
        },
        ExprKind::Assign { place, value } => ExprKind::Assign { place: bx(c, place)?, value: bx(c, value)? },
        ExprKind::Block(items) => ExprKind::Block(items.into_iter().map(|x| zonk_expr(c, x)).collect::<PResult<_>>()?),
        ExprKind::If { cond, then, els } => ExprKind::If {
            cond: bx(c, cond)?,
            then: bx(c, then)?,
            els: match els {
                Some(x) => Some(bx(c, x)?),
                None => None,
            },
        },
        ExprKind::While { cond, body } => ExprKind::While { cond: bx(c, cond)?, body: bx(c, body)? },
        ExprKind::Loop(b) => ExprKind::Loop(bx(c, b)?),
        ExprKind::For { local, lo, hi, body } => ExprKind::For { local, lo: bx(c, lo)?, hi: bx(c, hi)?, body: bx(c, body)? },
        ExprKind::Return(v) => ExprKind::Return(match v {
            Some(x) => Some(bx(c, x)?),
            None => None,
        }),
        ExprKind::Match { scrut, arms, by_ref } => ExprKind::Match {
            scrut: bx(c, scrut)?,
            arms: arms
                .into_iter()
                .map(|a| Ok(Arm { pat: a.pat, body: zonk_expr(c, a.body)?, span: a.span }))
                .collect::<PResult<_>>()?,
            by_ref,
        },
        ExprKind::Ref { mutable, place } => ExprKind::Ref { mutable, place: bx(c, place)? },
        ExprKind::Deref(x) => ExprKind::Deref(bx(c, x)?),
        ExprKind::Field { base, idx } => ExprKind::Field { base: bx(c, base)?, idx },
        ExprKind::Cast(x) => ExprKind::Cast(bx(c, x)?),
        ExprKind::Bin { op, l, r } => ExprKind::Bin { op, l: bx(c, l)?, r: bx(c, r)? },
        ExprKind::Neg(x) => ExprKind::Neg(bx(c, x)?),
        ExprKind::WNeg(x) => ExprKind::WNeg(bx(c, x)?),
        ExprKind::Not(x) => ExprKind::Not(bx(c, x)?),
        ExprKind::And(l, r) => ExprKind::And(bx(c, l)?, bx(c, r)?),
        ExprKind::Or(l, r) => ExprKind::Or(bx(c, l)?, bx(c, r)?),
        ExprKind::Call { callee, args } => {
            let callee = match callee {
                Callee::Fn(d, targs) => Callee::Fn(d, zonk_tys(c, &targs, span)?),
                Callee::Ptr(x) => Callee::Ptr(bx(c, x)?),
                Callee::Builtin(b) => Callee::Builtin(b),
                Callee::Sizeof(t) => Callee::Sizeof(zonk_tys(c, &[t], span)?.remove(0)),
            };
            ExprKind::Call { callee, args: args.into_iter().map(|x| zonk_expr(c, x)).collect::<PResult<_>>()? }
        }
        ExprKind::Struct { def, fields } => ExprKind::Struct { def, fields: fields.into_iter().map(|x| zonk_expr(c, x)).collect::<PResult<_>>()? },
        ExprKind::Variant { def, idx, fields } => ExprKind::Variant { def, idx, fields: fields.into_iter().map(|x| zonk_expr(c, x)).collect::<PResult<_>>()? },
        ExprKind::Try { expr, is_res } => ExprKind::Try { expr: bx(c, expr)?, is_res },
    };
    Ok(Expr { kind, ty, span })
}
