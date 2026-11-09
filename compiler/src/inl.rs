//! Reject `(inline ...)` when the body cannot be substituted.
//! Unmarked bodies of at most [`INLINE_LIMIT`] expression nodes are substituted too.
use crate::ast::{self, ExprKind};
use crate::diag::Diag;

/// Expression nodes, counted the same way `self/inline.llr` walks `expr` kids.
/// 0 keeps substitution on `(inline ...)` only. Raising it inlines `vec.len`
/// (`(. self len)`, no call, under 12 nodes) and `vec.sort` then reports
/// `borrow.mut_shared|ref=n`: the inlined result keeps `self`'s loan.
/// `Eq.eq` on `i64` is 5 nodes and `Hash.hash` on `i64` is 8; `fib` has neither.
pub const INLINE_LIMIT: usize = 0;

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

/// True when an unmarked, non-extern body should be substituted.
/// A call in the body stays a call: inlining it leaves the frame and an
/// unreachable block the linter reports. `Hash.hash` and `Eq.eq` have none.
pub fn small_body(xs: &[ast::Expr]) -> bool {
    !has_clos(xs) && !has_call(xs) && {
        let n = body_nodes(xs);
        n > 0 && n <= INLINE_LIMIT
    }
}

fn has_call(xs: &[ast::Expr]) -> bool {
    xs.iter().any(expr_call)
}

fn expr_call(e: &ast::Expr) -> bool {
    match &e.kind {
        ExprKind::Call { .. } | ExprKind::CallPtr { .. } => true,
        ExprKind::Let { init, .. } => init.as_deref().map(expr_call).unwrap_or(false),
        ExprKind::Set { place, value } | ExprKind::Bin { l: place, r: value, .. } | ExprKind::And(place, value) | ExprKind::Or(place, value) | ExprKind::AssertEq(place, value) | ExprKind::AssertNe(place, value) => {
            expr_call(place) || expr_call(value)
        }
        ExprKind::AssertNear(a, b, c) => expr_call(a) || expr_call(b) || expr_call(c),
        ExprKind::Do(xs) | ExprKind::Loop { body: xs, .. } => has_call(xs),
        ExprKind::While { cond, body } => expr_call(cond) || has_call(body),
        ExprKind::For { lo, hi, step, body, .. } => expr_call(lo) || expr_call(hi) || step.as_ref().is_some_and(|s| expr_call(s)) || has_call(body),
        ExprKind::If { cond, then, els } => expr_call(cond) || expr_call(then) || els.as_deref().map(expr_call).unwrap_or(false),
        ExprKind::Return(v) => v.as_deref().map(expr_call).unwrap_or(false),
        ExprKind::Match { scrut, arms } => expr_call(scrut) || arms.iter().any(|a| a.guard.as_ref().is_some_and(expr_call) || expr_call(&a.body)),
        ExprKind::Ref { place, .. } | ExprKind::Deref(place) | ExprKind::Not(place) | ExprKind::Neg(place) | ExprKind::WNeg(place) | ExprKind::BNot(place) | ExprKind::Try(place) | ExprKind::Panic(place) | ExprKind::Assert(place) | ExprKind::Drop(place) | ExprKind::Field { base: place, .. } | ExprKind::Cast { expr: place, .. } => expr_call(place),
        ExprKind::Clos { body, .. } => has_call(body),
        ExprKind::Lit(_) | ExprKind::Name(_) | ExprKind::Break(_) | ExprKind::Continue(_) | ExprKind::Sizeof(_) => false,
    }
}

pub fn body_nodes(xs: &[ast::Expr]) -> usize {
    let mut n = 0;
    for e in xs {
        let c = expr_nodes(e);
        if c >= 1_000_000 {
            return c;
        }
        n += c;
    }
    n
}

fn expr_nodes(e: &ast::Expr) -> usize {
    match &e.kind {
        // Match arms store patterns that the self AST counts as exprs. Bail so both sides agree.
        ExprKind::Match { .. } => 1_000_000,
        ExprKind::Lit(_) | ExprKind::Name(_) | ExprKind::Break(_) | ExprKind::Continue(_) | ExprKind::Sizeof(_) => 1,
        ExprKind::Let { init, .. } => 1 + init.as_deref().map(expr_nodes).unwrap_or(0),
        ExprKind::Set { place, value } | ExprKind::Bin { l: place, r: value, .. } | ExprKind::AssertEq(place, value) | ExprKind::AssertNe(place, value) => {
            1 + expr_nodes(place) + expr_nodes(value)
        }
        ExprKind::And(a, b) | ExprKind::Or(a, b) => 1 + expr_nodes(a) + expr_nodes(b),
        ExprKind::AssertNear(a, b, c) => 1 + expr_nodes(a) + expr_nodes(b) + expr_nodes(c),
        ExprKind::Do(xs) | ExprKind::Loop { body: xs, .. } => 1 + body_nodes(xs),
        ExprKind::While { cond, body } => 1 + expr_nodes(cond) + body_nodes(body),
        ExprKind::For { lo, hi, step, body, .. } => {
            1 + expr_nodes(lo) + expr_nodes(hi) + step.as_deref().map(expr_nodes).unwrap_or(0) + body_nodes(body)
        }
        ExprKind::If { cond, then, els } => 1 + expr_nodes(cond) + expr_nodes(then) + els.as_deref().map(expr_nodes).unwrap_or(0),
        ExprKind::Return(v) => 1 + v.as_deref().map(expr_nodes).unwrap_or(0),
        ExprKind::Ref { place: v, .. } | ExprKind::Deref(v) | ExprKind::Not(v) | ExprKind::Neg(v) | ExprKind::WNeg(v) | ExprKind::BNot(v) | ExprKind::Try(v) | ExprKind::Panic(v) | ExprKind::Assert(v) | ExprKind::Drop(v) | ExprKind::Field { base: v, .. } | ExprKind::Cast { expr: v, .. } => {
            1 + expr_nodes(v)
        }
        ExprKind::Call { args, .. } => 1 + body_nodes(args),
        ExprKind::CallPtr { callee, args } => 1 + expr_nodes(callee) + body_nodes(args),
        ExprKind::Clos { body, .. } => 1 + body_nodes(body),
    }
}

fn has_clos(xs: &[ast::Expr]) -> bool {
    xs.iter().any(expr_clos)
}

fn expr_clos(e: &ast::Expr) -> bool {
    match &e.kind {
        ExprKind::Clos { .. } => true,
        ExprKind::Let { init, .. } => init.as_deref().map(expr_clos).unwrap_or(false),
        ExprKind::Set { place, value } | ExprKind::Bin { l: place, r: value, .. } | ExprKind::And(place, value) | ExprKind::Or(place, value) | ExprKind::AssertEq(place, value) | ExprKind::AssertNe(place, value) => expr_clos(place) || expr_clos(value),
        ExprKind::AssertNear(a, b, c) => expr_clos(a) || expr_clos(b) || expr_clos(c),
        ExprKind::Do(xs) => has_clos(xs),
        ExprKind::Loop { body, .. } => has_clos(body),
        ExprKind::While { cond, body } => expr_clos(cond) || has_clos(body),
        ExprKind::For { lo, hi, step, body, .. } => {
            expr_clos(lo) || expr_clos(hi) || step.as_ref().is_some_and(|s| expr_clos(s)) || has_clos(body)
        }
        ExprKind::If { cond, then, els } => expr_clos(cond) || expr_clos(then) || els.as_deref().map(expr_clos).unwrap_or(false),
        ExprKind::Return(v) => v.as_deref().map(expr_clos).unwrap_or(false),
        ExprKind::Match { scrut, arms } => {
            expr_clos(scrut) || arms.iter().any(|a| a.guard.as_ref().is_some_and(expr_clos) || expr_clos(&a.body))
        }
        ExprKind::Ref { place, .. } | ExprKind::Deref(place) | ExprKind::Not(place) | ExprKind::Neg(place) | ExprKind::WNeg(place) | ExprKind::BNot(place) | ExprKind::Try(place) | ExprKind::Panic(place) | ExprKind::Assert(place) | ExprKind::Drop(place) | ExprKind::Field { base: place, .. } | ExprKind::Cast { expr: place, .. } => expr_clos(place),
        ExprKind::Call { args, .. } => has_clos(args),
        ExprKind::CallPtr { callee, args } => expr_clos(callee) || has_clos(args),
        ExprKind::Break(_) | ExprKind::Continue(_) | ExprKind::Lit(_) | ExprKind::Name(_) | ExprKind::Sizeof(_) => false,
    }
}
