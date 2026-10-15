//! Backward liveness of locals at every MIR point.
use super::*;
use crate::mir::needs_drop;

pub struct Liveness {
    /// `at[bb][idx]` = locals live immediately before point (bb, idx); idx == stmts.len()
    /// is the terminator. `out[bb]` = live after the terminator.
    pub at: Vec<Vec<BitSet>>,
    pub out: Vec<BitSet>,
}

impl Liveness {
    pub fn live_at(&self, p: Point, local: LocalIdx) -> bool {
        self.at[p.bb as usize][p.idx].contains(local)
    }
    pub fn live_after(&self, p: Point, local: LocalIdx) -> bool {
        let blk = &self.at[p.bb as usize];
        if p.idx + 1 < blk.len() {
            blk[p.idx + 1].contains(local)
        } else {
            self.out[p.bb as usize].contains(local)
        }
    }
}

/// (uses, defs) of one set of accesses. Writes to a whole local define it; writes through
/// projections both use and define.
fn gen_kill(prog: &Program, body: &Body, accs: &[Access], uses: &mut BitSet, defs: &mut BitSet) {
    for a in accs {
        match a.kind {
            AccessKind::Read | AccessKind::Move | AccessKind::BorrowShared | AccessKind::BorrowMut => {
                uses.insert(a.place.local);
            }
            AccessKind::Write => {
                if a.place.is_local() {
                    defs.insert(a.place.local);
                } else {
                    uses.insert(a.place.local);
                }
            }
            AccessKind::Drop => {
                if a.place.is_local() {
                    if needs_drop(prog, body.local_ty(a.place.local)) {
                        uses.insert(a.place.local);
                    }
                } else {
                    uses.insert(a.place.local);
                }
            }
            AccessKind::StorageDead => {
                defs.insert(a.place.local);
            }
        }
    }
}

pub fn compute(prog: &Program, body: &Body) -> Liveness {
    let n = body.locals.len();
    let nb = body.blocks.len();
    let mut live_in: Vec<BitSet> = (0..nb).map(|_| BitSet::new(n)).collect();
    let mut live_out: Vec<BitSet> = (0..nb).map(|_| BitSet::new(n)).collect();
    // Per-block transfer: walk backwards once per iteration.
    let mut changed = true;
    while changed {
        changed = false;
        for bb in (0..nb).rev() {
            let mut out = BitSet::new(n);
            for s in body.successors(bb as BlockId) {
                out.union_with(&live_in[s as usize]);
            }
            let cur = transfer_block(prog, body, bb as BlockId, &out, None);
            if out != live_out[bb] {
                live_out[bb] = out;
                changed = true;
            }
            if cur != live_in[bb] {
                live_in[bb] = cur;
                changed = true;
            }
        }
    }
    let mut at = Vec::with_capacity(nb);
    for bb in 0..nb {
        let mut per_point = Vec::new();
        transfer_block(prog, body, bb as BlockId, &live_out[bb], Some(&mut per_point));
        at.push(per_point);
    }
    Liveness { at, out: live_out }
}

/// Computes live-in for a block from live-out; optionally records per-point sets.
fn transfer_block(prog: &Program, body: &Body, bb: BlockId, out: &BitSet, mut record: Option<&mut Vec<BitSet>>) -> BitSet {
    let n = body.locals.len();
    let blk = &body.blocks[bb as usize];
    let mut live = out.clone();
    let mut points: Vec<BitSet> = Vec::new();
    let accs = term_accesses(&blk.term, blk.term_span);
    apply(prog, body, &accs, &mut live, n);
    points.push(live.clone());
    for (s, sp) in blk.stmts.iter().rev() {
        let accs = stmt_accesses(s, *sp);
        apply(prog, body, &accs, &mut live, n);
        points.push(live.clone());
    }
    if let Some(r) = record.as_mut() {
        points.reverse();
        **r = points;
    }
    live
}

fn apply(prog: &Program, body: &Body, accs: &[Access], live: &mut BitSet, n: usize) {
    let mut uses = BitSet::new(n);
    let mut defs = BitSet::new(n);
    gen_kill(prog, body, accs, &mut uses, &mut defs);
    for d in defs.iter() {
        live.remove(d);
    }
    live.union_with(&uses);
}
