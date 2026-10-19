//! Reject `(inline ...)` when the body cannot be substituted.
use crate::ast::{self, ExprKind};
use crate::diag::Diag;

pub fn inline_reject(f: &ast::FnDecl) -> Option<Diag> {
    if !f.inline {
        return None;
    }
    if f.is_extern {
        return Some(Diag::error(f.span, "inline.reject").kv("why", "extern"));
    }
    if has_clos(&f.body) {
        return Some(Diag::error(f.span, "inline.reject").kv("why", "form").kv("got", "clos"));
    }
    None
}

fn has_clos(xs: &[ast::Expr]) -> bool {
    xs.iter().any(expr_clos)
}

fn expr_clos(e: &ast::Expr) -> bool {
    match &e.kind {
        ExprKind::Clos { .. } => true,
        ExprKind::Let { init, .. } => init.as_deref().map(expr_clos).unwrap_or(false),
        ExprKind::Set { place, value } | ExprKind::Bin { l: place, r: value, .. } | ExprKind::And(place, value) | ExprKind::Or(place, value) | ExprKind::AssertEq(place, value) => expr_clos(place) || expr_clos(value),
        ExprKind::Do(xs) | ExprKind::Loop(xs) => has_clos(xs),
        ExprKind::While { cond, body } => expr_clos(cond) || has_clos(body),
        ExprKind::For { lo, hi, body, .. } => expr_clos(lo) || expr_clos(hi) || has_clos(body),
        ExprKind::If { cond, then, els } => expr_clos(cond) || expr_clos(then) || els.as_deref().map(expr_clos).unwrap_or(false),
        ExprKind::Return(v) => v.as_deref().map(expr_clos).unwrap_or(false),
        ExprKind::Match { scrut, arms } => expr_clos(scrut) || arms.iter().any(|a| expr_clos(&a.body)),
        ExprKind::Ref { place, .. } | ExprKind::Deref(place) | ExprKind::Not(place) | ExprKind::Neg(place) | ExprKind::WNeg(place) | ExprKind::Try(place) | ExprKind::Panic(place) | ExprKind::Assert(place) | ExprKind::Drop(place) | ExprKind::Field { base: place, .. } | ExprKind::Cast { expr: place, .. } => expr_clos(place),
        ExprKind::Call { args, .. } => has_clos(args),
        ExprKind::CallPtr { callee, args } => expr_clos(callee) || has_clos(args),
        ExprKind::Break | ExprKind::Continue | ExprKind::Lit(_) | ExprKind::Name(_) | ExprKind::Sizeof(_) => false,
    }
}
