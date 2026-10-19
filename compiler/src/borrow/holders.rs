//! Flow-sensitive holder sets: at every point, which locals may hold a reference derived
//! from which loan. Forward dataflow over (loan, local) pairs, union at joins. A whole-local
//! write kills every pair of that local, so reassigning a reference releases old loans.
use super::liveness::Liveness;
use super::loans::{eligible_args, mut_ref_sink_args, Loan};
use super::*;
use crate::diag::{Diag, Diags};
use crate::mir::place_ty;
use crate::ty::Ty;
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

#[derive(Clone)]
struct Flow {
    bits: BitSet,
    /// Loan ids stored in each `(local, field)`.
    fields: HashMap<(LocalIdx, u32), Vec<usize>>,
}

impl Flow {
    fn union_with(&mut self, o: &Flow) -> bool {
        let bits = self.bits.union_with(&o.bits);
        let mut extra = false;
        for (k, vs) in &o.fields {
            let e = self.fields.entry(*k).or_default();
            for loan in vs {
                if !e.contains(loan) {
                    e.push(*loan);
                    extra = true;
                }
            }
        }
        bits || extra
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

    fn kill_field(&self, st: &mut Flow, local: LocalIdx, field: u32) {
        let Some(gone) = st.fields.remove(&(local, field)) else { return };
        for li in gone {
            let held = st.fields.iter().any(|((l, _), vs)| *l == local && vs.contains(&li));
            if !held {
                st.bits.remove(self.bit(li, local));
            }
        }
    }

    fn note_field(&self, st: &mut Flow, local: LocalIdx, field: u32, loans: &[usize]) {
        if loans.is_empty() {
            return;
        }
        let e = st.fields.entry((local, field)).or_default();
        for li in loans {
            if !e.contains(li) {
                e.push(*li);
            }
        }
    }

    fn clear_fields(&self, st: &mut Flow, local: LocalIdx) {
        st.fields.retain(|(l, _), _| *l != local);
    }

    fn loans_of(&self, pre: &BitSet, rv: &Rvalue, to: LocalIdx, point: Point) -> Vec<usize> {
        let mut gen = Vec::new();
        match rv {
            Rvalue::Ref { place, .. } => {
                if let Some(&li) = self.loan_at.get(&point) {
                    gen.push(li);
                }
                gen.extend(self.pairs_of(pre, place.local));
            }
            Rvalue::Use(op) | Rvalue::Cast(op, _) => gen.extend(self.flow_operand(pre, op, to)),
            Rvalue::Struct(_, ops) | Rvalue::Variant(_, _, ops) | Rvalue::Closure { caps: ops, .. } => {
                for o in ops {
                    gen.extend(self.flow_operand(pre, o, to));
                }
            }
            _ => {}
        }
        gen
    }

    fn stmt(&self, st: &mut Flow, s: &Stmt, point: Point) {
        match s {
            Stmt::Assign(dest, rv) => {
                if dest.has_deref() {
                    let gen = self.loans_of(&st.bits, rv, dest.local, point);
                    self.add_all(&mut st.bits, &gen, dest.local);
                    return;
                }
                if let Some(field) = dest.lone_field() {
                    let gen = self.loans_of(&st.bits.clone(), rv, dest.local, point);
                    self.kill_field(st, dest.local, field);
                    self.note_field(st, dest.local, field, &gen);
                    self.add_all(&mut st.bits, &gen, dest.local);
                    return;
                }
                if dest.is_local() {
                    let pre = st.bits.clone();
                    self.kill_local(&mut st.bits, dest.local);
                    self.clear_fields(st, dest.local);
                    match rv {
                        Rvalue::Struct(_, ops) | Rvalue::Variant(_, _, ops) | Rvalue::Closure { caps: ops, .. } => {
                            for (i, o) in ops.iter().enumerate() {
                                let g = self.flow_operand(&pre, o, dest.local);
                                self.note_field(st, dest.local, i as u32, &g);
                                self.add_all(&mut st.bits, &g, dest.local);
                            }
                        }
                        _ => {
                            let gen = self.loans_of(&pre, rv, dest.local, point);
                            self.add_all(&mut st.bits, &gen, dest.local);
                        }
                    }
                    return;
                }
                let gen = self.loans_of(&st.bits, rv, dest.local, point);
                self.add_all(&mut st.bits, &gen, dest.local);
            }
            Stmt::StorageDead(l) => {
                self.kill_local(&mut st.bits, *l);
                self.clear_fields(st, *l);
            }
            Stmt::Drop(_) | Stmt::Forget(_) | Stmt::Nop => {}
            Stmt::InlineLoans { kill_from, sources, dest } => {
                for li in 0..self.loans.len() {
                    if self.loans[li].place.local >= *kill_from {
                        for loc in 0..self.nloc as LocalIdx {
                            st.bits.remove(self.bit(li, loc));
                        }
                        for vs in st.fields.values_mut() {
                            vs.retain(|x| *x != li);
                        }
                    }
                }
                let mut gen = Vec::new();
                for s in sources {
                    gen.extend(self.pairs_of(&st.bits, *s));
                }
                self.add_all(&mut st.bits, &gen, *dest);
            }
        }
    }

    fn term(&self, st: &mut Flow, t: &Term) {
        let Term::Call { func, args, dest, .. } = t else { return };
        let pre = st.bits.clone();
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
            self.kill_local(&mut st.bits, dest.local);
            self.clear_fields(st, dest.local);
        }
        self.add_all(&mut st.bits, &gen, dest.local);
        for (sink, stored) in sinks {
            self.add_all(&mut st.bits, &stored, sink);
        }
    }

    fn transfer_block(&self, bb: BlockId, input: &Flow, mut record: Option<&mut Vec<BitSet>>) -> Flow {
        let blk = &self.body.blocks[bb as usize];
        let mut st = input.clone();
        for (si, (s, _)) in blk.stmts.iter().enumerate() {
            if let Some(r) = record.as_mut() {
                r.push(st.bits.clone());
            }
            self.stmt(&mut st, s, Point { bb, idx: si });
        }
        if let Some(r) = record.as_mut() {
            r.push(st.bits.clone());
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
    let mut inp: Vec<Flow> = (0..nb).map(|_| Flow { bits: cx.new_state(), fields: HashMap::new() }).collect();
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

/// A local is static when it holds a `'static` pointer: a string literal, the address of
/// a const, a copy of one of those, a `'static` parameter, or a call that returns `'static`.
pub fn static_locals(diags: &Diags, body: &Body) -> Vec<bool> {
    let n = body.locals.len();
    let mut bad = vec![false; n];
    let mut seen = vec![false; n];
    let mut src: Vec<Vec<Option<LocalIdx>>> = (0..n).map(|_| Vec::new()).collect();
    let mut wrote: Vec<Option<Span>> = vec![None; n];
    for i in 1..=body.arg_count.min(n.saturating_sub(1)) {
        if is_static_ref(&body.locals[i].ty) {
            seen[i] = true;
            src[i].push(None);
        }
    }
    for blk in &body.blocks {
        for (s, sp) in &blk.stmts {
            let Stmt::Assign(dest, rv) = s else { continue };
            if !dest.is_local() {
                continue;
            }
            note_write(body, &mut bad, &mut seen, &mut src, &mut wrote, dest.local, rv, *sp);
        }
        if let Term::Call { dest, .. } = &blk.term {
            if dest.is_local() {
                let i = dest.local as usize;
                seen[i] = true;
                if wrote[i].is_none() {
                    wrote[i] = Some(blk.term_span);
                }
                if is_static_ref(body.local_ty(dest.local)) {
                    src[i].push(None);
                } else {
                    bad[i] = true;
                }
            }
        }
    }
    let mut stat = vec![false; n];
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..n {
            if bad[i] || !seen[i] || stat[i] {
                continue;
            }
            let ok = src[i].iter().all(|s| match s {
                None => true,
                Some(l) => stat[*l as usize],
            });
            if ok {
                stat[i] = true;
                changed = true;
            }
        }
    }
    for i in 0..n {
        if is_static_ref(&body.locals[i].ty) && !stat[i] {
            let sp = wrote[i].unwrap_or(body.locals[i].span);
            diags.push(Diag::error(sp, "borrow.escape").kv("lt", "static"));
        }
    }
    stat
}

fn note_write(body: &Body, bad: &mut [bool], seen: &mut [bool], src: &mut [Vec<Option<LocalIdx>>], wrote: &mut [Option<Span>], local: LocalIdx, rv: &Rvalue, sp: Span) {
    let i = local as usize;
    seen[i] = true;
    if wrote[i].is_none() {
        wrote[i] = Some(sp);
    }
    let static_dest = is_static_ref(body.local_ty(local));
    match rv {
        Rvalue::Use(Operand::Const(Const::Str(_), _)) | Rvalue::Static(_) => src[i].push(None),
        Rvalue::Use(Operand::Copy(p) | Operand::Move(p)) if p.is_local() => src[i].push(Some(p.local)),
        Rvalue::Use(Operand::Copy(_) | Operand::Move(_)) if static_dest => src[i].push(None),
        _ => bad[i] = true,
    }
}

fn is_static_ref(t: &Ty) -> bool {
    match t {
        Ty::Ref { lt: Some(l), .. } => l == "static" || l == "'static",
        _ => false,
    }
}
