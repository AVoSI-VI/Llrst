//! Flow-sensitive holder sets: at every point, which locals may hold a reference derived
//! from which loan. Forward dataflow over (loan, local) pairs, union at joins. A whole-local
//! write kills every pair of that local, so reassigning a reference releases old loans.
use super::liveness::Liveness;
use super::loans::{eligible_args, mut_ref_sink_args, Loan};
use super::*;
use crate::mir::place_ty;
use std::collections::HashMap;

pub struct Holders {
    nloc: usize,
    /// `at[bb][idx]` = pairs held immediately before point (bb, idx); idx == stmts.len() is
    /// the terminator.
    at: Vec<Vec<BitSet>>,
}

impl Holders {
    fn bit(&self, loan: usize, local: LocalIdx) -> u32 {
        (loan * self.nloc + local as usize) as u32
    }
    pub fn held(&self, p: Point, loan: usize, local: LocalIdx) -> bool {
        self.at[p.bb as usize][p.idx].contains(self.bit(loan, local))
    }
    fn holders_at(&self, p: Point, loan: usize) -> impl Iterator<Item = LocalIdx> + '_ {
        (0..self.nloc as LocalIdx).filter(move |l| self.held(p, loan, *l))
    }
    /// Some holder of the loan is live at the point.
    pub fn in_force(&self, p: Point, loan: usize, live: &Liveness) -> bool {
        self.holders_at(p, loan).any(|h| live.live_at(p, h))
    }
    /// Some holder of the loan is live after the point (the referent dies first).
    pub fn any_holder_live_after(&self, p: Point, loan: usize, live: &Liveness) -> bool {
        self.holders_at(p, loan).any(|h| live.live_after(p, h))
    }
    /// A readable holder name for diagnostics: first non-temporary local.
    pub fn some_holder_name(&self, p: Point, loan: usize, body: &Body) -> String {
        self.holders_at(p, loan)
            .map(|h| body.locals[h as usize].name.clone())
            .find(|n| !n.starts_with("_t"))
            .unwrap_or_default()
    }
}

struct Cx<'a> {
    prog: &'a Program,
    body: &'a Body,
    loans: &'a [Loan],
    loan_at: HashMap<Point, usize>,
    nloc: usize,
}

impl<'a> Cx<'a> {
    fn bit(&self, loan: usize, local: LocalIdx) -> u32 {
        (loan * self.nloc + local as usize) as u32
    }
    fn new_state(&self) -> BitSet {
        BitSet::new(self.loans.len() * self.nloc)
    }
    fn pairs_of(&self, st: &BitSet, local: LocalIdx) -> Vec<usize> {
        (0..self.loans.len()).filter(|l| st.contains(self.bit(*l, local))).collect()
    }
    fn kill_local(&self, st: &mut BitSet, local: LocalIdx) {
        for l in 0..self.loans.len() {
            st.remove(self.bit(l, local));
        }
    }
    fn add_all(&self, st: &mut BitSet, loans: &[usize], to: LocalIdx) {
        for l in loans {
            st.insert(self.bit(*l, to));
        }
    }
    fn ref_bearing(&self, local: LocalIdx) -> bool {
        self.body.local_ty(local).has_ref()
    }
    /// Pairs that flow from an operand into a destination local.
    fn flow_operand(&self, pre: &BitSet, op: &Operand, to: LocalIdx) -> Vec<usize> {
        match op.place() {
            Some(p) if self.ref_bearing(p.local) || self.ref_bearing(to) => self.pairs_of(pre, p.local),
            _ => Vec::new(),
        }
    }

    fn stmt(&self, st: &mut BitSet, s: &Stmt, point: Point) {
        match s {
            Stmt::Assign(dest, rv) => {
                let pre = st.clone();
                let to = dest.local;
                // reads before the write
                let mut gen: Vec<usize> = Vec::new();
                match rv {
                    Rvalue::Ref { place, .. } => {
                        if let Some(&li) = self.loan_at.get(&point) {
                            gen.push(li);
                        }
                        // A reference into a holder can reach everything the holder holds.
                        gen.extend(self.pairs_of(&pre, place.local));
                    }
                    Rvalue::Use(op) | Rvalue::Cast(op, _) => gen.extend(self.flow_operand(&pre, op, to)),
                    Rvalue::Struct(_, ops) | Rvalue::Variant(_, _, ops) => {
                        for o in ops {
                            gen.extend(self.flow_operand(&pre, o, to));
                        }
                    }
                    _ => {}
                }
                if dest.is_local() {
                    self.kill_local(st, to);
                }
                self.add_all(st, &gen, to);
            }
            Stmt::StorageDead(l) => self.kill_local(st, *l),
            Stmt::Drop(_) | Stmt::Forget(_) | Stmt::Nop => {}
            Stmt::InlineLoans { kill_from, sources, dest } => {
                for li in 0..self.loans.len() {
                    if self.loans[li].place.local >= *kill_from {
                        for loc in 0..self.nloc as LocalIdx {
                            st.remove(self.bit(li, loc));
                        }
                    }
                }
                let mut gen = Vec::new();
                for s in sources {
                    gen.extend(self.pairs_of(st, *s));
                }
                self.add_all(st, &gen, *dest);
            }
        }
    }

    fn term(&self, st: &mut BitSet, t: &Term) {
        let Term::Call { func, args, dest, .. } = t else { return };
        let pre = st.clone();
        let roots: Vec<Option<LocalIdx>> = args.iter().map(|a| a.place().map(|p| p.local)).collect();
        // Result derives from the eligible arguments.
        let mut gen: Vec<usize> = Vec::new();
        let dest_has_ref = place_ty(self.prog, self.body, dest).has_ref() || self.ref_bearing(dest.local);
        if dest_has_ref {
            for i in eligible_args(self.prog, func, args, self.body) {
                if let Some(r) = roots[i] {
                    gen.extend(self.pairs_of(&pre, r));
                }
            }
        }
        // References passed alongside a `(&mut container)` may be stored in the container:
        // every referent the `&mut` loans point at becomes a holder of those references.
        let mut sinks: Vec<(LocalIdx, Vec<usize>)> = Vec::new();
        for i in mut_ref_sink_args(self.prog, func, args, self.body) {
            let Some(via) = roots[i] else { continue };
            let mut stored: Vec<usize> = Vec::new();
            for (j, r) in roots.iter().enumerate() {
                if j == i {
                    continue;
                }
                if let Some(r) = r {
                    if self.ref_bearing(*r) {
                        stored.extend(self.pairs_of(&pre, *r));
                    }
                }
            }
            if stored.is_empty() {
                continue;
            }
            for lm in self.pairs_of(&pre, via) {
                if self.loans[lm].mutable {
                    sinks.push((self.loans[lm].place.local, stored.clone()));
                }
            }
        }
        if dest.is_local() {
            self.kill_local(st, dest.local);
        }
        self.add_all(st, &gen, dest.local);
        for (sink, stored) in sinks {
            self.add_all(st, &stored, sink);
        }
    }

    fn transfer_block(&self, bb: BlockId, input: &BitSet, mut record: Option<&mut Vec<BitSet>>) -> BitSet {
        let blk = &self.body.blocks[bb as usize];
        let mut st = input.clone();
        for (si, (s, _)) in blk.stmts.iter().enumerate() {
            if let Some(r) = record.as_mut() {
                r.push(st.clone());
            }
            self.stmt(&mut st, s, Point { bb, idx: si });
        }
        if let Some(r) = record.as_mut() {
            r.push(st.clone());
        }
        self.term(&mut st, &blk.term);
        st
    }
}

pub fn compute(prog: &Program, body: &Body, loans: &[Loan]) -> Holders {
    let nloc = body.locals.len();
    let loan_at: HashMap<Point, usize> = loans.iter().enumerate().map(|(i, l)| (l.point, i)).collect();
    let cx = Cx { prog, body, loans, loan_at, nloc };
    let nb = body.blocks.len();
    let mut inp: Vec<BitSet> = (0..nb).map(|_| cx.new_state()).collect();
    let mut changed = true;
    while changed {
        changed = false;
        for bb in 0..nb {
            let out = cx.transfer_block(bb as BlockId, &inp[bb], None);
            for s in body.successors(bb as BlockId) {
                if inp[s as usize].union_with(&out) {
                    changed = true;
                }
            }
        }
    }
    let mut at = Vec::with_capacity(nb);
    for bb in 0..nb {
        let mut points = Vec::new();
        cx.transfer_block(bb as BlockId, &inp[bb], Some(&mut points));
        at.push(points);
    }
    Holders { nloc, at }
}
