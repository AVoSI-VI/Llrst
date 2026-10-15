//! Loan collection and region inference. A loan's region is the set of points where any
//! local that may hold a reference derived from it is live.
use super::liveness::Liveness;
use super::*;
use crate::ty::Ty;

#[derive(Clone, Debug)]
pub struct Loan {
    pub place: Place,
    pub mutable: bool,
    pub two_phase: bool,
    pub point: Point,
    pub holders: BitSet,
    pub span: Span,
}

impl Loan {
    pub fn live_at(&self, p: Point, live: &Liveness) -> bool {
        self.holders.iter().any(|h| live.live_at(p, h))
    }
    /// Two-phase loans act as shared until the call terminator of their block.
    pub fn active_at(&self, p: Point) -> bool {
        !self.two_phase || p.bb != self.point.bb
    }
}

/// Flow edge: references held by `from` may flow into `to`.
struct Flow {
    from: LocalIdx,
    to: LocalIdx,
}

pub fn compute(prog: &Program, body: &Body, live: &Liveness) -> Vec<Loan> {
    let _ = live;
    let n = body.locals.len();
    let mut loans: Vec<Loan> = Vec::new();
    let mut flows: Vec<Flow> = Vec::new();
    // (loan index, root local of the reborrowed reference)
    let mut reborrows: Vec<(usize, LocalIdx)> = Vec::new();
    for (bi, blk) in body.blocks.iter().enumerate() {
        for (si, (s, sp)) in blk.stmts.iter().enumerate() {
            let Stmt::Assign(dest, rv) = s else { continue };
            let point = Point { bb: bi as BlockId, idx: si };
            match rv {
                Rvalue::Ref { mutable, place, two_phase } => {
                    let mut holders = BitSet::new(n);
                    holders.insert(dest.local);
                    if place.has_deref() {
                        reborrows.push((loans.len(), place.local));
                    }
                    loans.push(Loan { place: place.clone(), mutable: *mutable, two_phase: *two_phase, point, holders, span: *sp });
                }
                Rvalue::Use(op) | Rvalue::Cast(op, _) => flow_operand(body, op, dest.local, &mut flows),
                Rvalue::Struct(_, ops) | Rvalue::Variant(_, _, ops) => ops.iter().for_each(|o| flow_operand(body, o, dest.local, &mut flows)),
                _ => {}
            }
        }
        if let Term::Call { func, args, dest, .. } = &blk.term {
            let dest_ty = body.local_ty(dest.local);
            if !crate::mir::place_ty(prog, body, dest).has_ref() && !dest_ty.has_ref() {
                continue;
            }
            for a in eligible_args(prog, func, args, body) {
                if let Some(p) = args[a].place() {
                    flows.push(Flow { from: p.local, to: dest.local });
                }
            }
        }
    }
    // Fixpoint over holder sets.
    let mut changed = true;
    while changed {
        changed = false;
        for f in &flows {
            for l in loans.iter_mut() {
                if l.holders.contains(f.from) && l.holders.insert(f.to) {
                    changed = true;
                }
            }
        }
        for &(li, root) in &reborrows {
            let inner = loans[li].holders.clone();
            for (j, l) in loans.iter_mut().enumerate() {
                if j != li && l.holders.contains(root) && l.holders.union_with(&inner) {
                    changed = true;
                }
            }
        }
    }
    loans
}

fn flow_operand(body: &Body, op: &Operand, to: LocalIdx, flows: &mut Vec<Flow>) {
    if let Some(p) = op.place() {
        if body.local_ty(p.local).has_ref() || body.local_ty(to).has_ref() {
            flows.push(Flow { from: p.local, to });
        }
    }
}

/// Indices of call arguments whose loans flow into a reference-bearing result, following
/// named lifetimes, then elision rules.
fn eligible_args(prog: &Program, func: &Func, args: &[Operand], body: &Body) -> Vec<usize> {
    let Func::Def(def, targs) = func else {
        return (0..args.len()).filter(|&i| arg_has_ref(body, &args[i])).collect();
    };
    let fd = prog.fn_def(*def);
    let ret = fd.ret.subst(targs);
    let params: Vec<Ty> = fd.params.iter().map(|p| p.1.subst(targs)).collect();
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

fn arg_has_ref(body: &Body, op: &Operand) -> bool {
    op.place().map_or(false, |p| body.local_ty(p.local).has_ref())
}
