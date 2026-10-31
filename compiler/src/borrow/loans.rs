//! Loan collection and the per-call rules for how references flow through signatures.
//! Who holds a loan at each point is computed in `holders.rs`.
use super::*;
use crate::mir::place_ty;
use crate::ty::Ty;

#[derive(Clone, Debug)]
pub struct Loan {
    pub place: Place,
    pub mutable: bool,
    pub two_phase: bool,
    pub point: Point,
    pub span: Span,
}

impl Loan {
    /// Two-phase loans act as shared until the call terminator of their block.
    pub fn active_at(&self, p: Point) -> bool {
        !self.two_phase || p.bb != self.point.bb
    }
}

/// Every `&`/`&mut` in program order; the index is the loan id.
pub fn compute(body: &Body) -> Vec<Loan> {
    let mut loans = Vec::new();
    for (bi, blk) in body.blocks.iter().enumerate() {
        for (si, (s, sp)) in blk.stmts.iter().enumerate() {
            if let Stmt::Assign(_, Rvalue::Ref { mutable, place, two_phase }) = s {
                let point = Point { bb: bi as BlockId, idx: si };
                loans.push(Loan { place: place.clone(), mutable: *mutable, two_phase: *two_phase, point, span: *sp });
            }
            if let Stmt::Assign(_, Rvalue::BoxBorrow { mutable, place }) = s {
                let point = Point { bb: bi as BlockId, idx: si };
                loans.push(Loan { place: place.clone(), mutable: *mutable, two_phase: false, point, span: *sp });
            }
        }
    }
    loans
}

/// Parameter types of a call target after substitution, when statically known.
fn param_tys(prog: &Program, func: &Func) -> Option<Vec<Ty>> {
    let Func::Def(def, targs) = func else { return None };
    Some(prog.fn_def(*def).params.iter().map(|p| p.1.subst(targs)).collect())
}

/// Indices of call arguments whose loans flow into a reference-bearing result, following
/// named lifetimes, then elision rules.
pub fn eligible_args(prog: &Program, func: &Func, args: &[Operand], body: &Body) -> Vec<usize> {
    let Func::Def(def, targs) = func else {
        return (0..args.len()).filter(|&i| arg_has_ref(body, &args[i])).collect();
    };
    let fd = prog.fn_def(*def);
    let ret = fd.ret.subst(targs);
    let params = param_tys(prog, func).unwrap();
    let mut ret_lts = Vec::new();
    ret.lifetimes(&mut ret_lts);
    if !ret_lts.is_empty() {
        return (0..params.len())
            .filter(|&i| {
                let mut l = Vec::new();
                params[i].lifetimes(&mut l);
                l.iter().any(|x| ret_lts.contains(x))
            })
            .collect();
    }
    let refs: Vec<usize> = (0..params.len()).filter(|&i| params[i].has_ref()).collect();
    if refs.len() == 1 {
        return refs;
    }
    if let Some(first) = fd.params.first() {
        if first.0 == "self" && params[0].has_ref() {
            return vec![0];
        }
    }
    refs
}

/// Indices of arguments typed `(&mut X)` where `X` can hold references: the callee may
/// store any other reference argument inside `X`.
pub fn mut_ref_sink_args(prog: &Program, func: &Func, args: &[Operand], body: &Body) -> Vec<usize> {
    let tys: Vec<Ty> = match param_tys(prog, func) {
        Some(t) => t,
        None => args.iter().map(|a| a.place().map(|p| place_ty(prog, body, p)).unwrap_or(Ty::Unit)).collect(),
    };
    (0..args.len().min(tys.len()))
        .filter(|&i| matches!(&tys[i], Ty::Ref { mutable: true, inner, .. } if inner.has_ref()))
        .collect()
}

fn arg_has_ref(body: &Body, op: &Operand) -> bool {
    op.place().map_or(false, |p| body.local_ty(p.local).has_ref())
}
