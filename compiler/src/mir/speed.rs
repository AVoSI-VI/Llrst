//! After borrowck, before codegen. `emit-mir` stays the unoptimized body.
//! Inline frames are pushed only around calls, and only when the inlined region
//! is not a loop: a straight-line `vec.get` pays for the frame on the panic path.
//! A `std` `clone` or `from` of a local that dies at the call becomes a move.
//! A drop whose flag is definitely clear is removed.
use super::*;
use crate::borrow::{compute_holders, compute_live, compute_loans, stmt_accesses, term_accesses, AccessKind, Loan, Point};
use crate::resolve::Program;
use crate::ty::{DefId, Ty};

pub fn run(prog: &Program, body: &mut Body) {
    elide_clones(prog, body);
    elide_traces(body);
    elide_drops(body);
}

// ----- clone to move -----

fn elide_clones(prog: &Program, body: &mut Body) {
    let mut edits: Vec<(usize, Place, LocalIdx, BlockId, Span)> = Vec::new();
    let mut saw = false;
    for blk in &body.blocks {
        if let Term::Call { func: Func::Def(id, _), args, .. } = &blk.term {
            if args.len() == 1 && is_std_copy(prog, *id) {
                saw = true;
            }
        }
    }
    if !saw {
        return;
    }
    let loans = compute_loans(body);
    let live = compute_live(prog, body);
    let holders = compute_holders(prog, body, &loans);
    for (bi, blk) in body.blocks.iter().enumerate() {
        let Term::Call { func: Func::Def(id, targs), args, dest, next, span } = &blk.term else { continue };
        if args.len() != 1 || !is_std_copy(prog, *id) {
            continue;
        }
        let Some(arg) = whole_local(&args[0]) else { continue };
        let Some((src, loan_at)) = owned_ref(body, arg) else { continue };
        if dest.local == src {
            continue;
        }
        if !sig_matches(prog, *id, targs, body.local_ty(src)) {
            continue;
        }
        if used_later(body, bi, src) {
            continue;
        }
        let Some(li) = loan_index(&loans, loan_at) else { continue };
        let at = Point { bb: bi as BlockId, idx: blk.stmts.len() };
        if holders.any_holder_live_after(at, li, &live) {
            continue;
        }
        edits.push((bi, dest.clone(), src, *next, *span));
    }
    for (bi, dest, src, next, span) in edits {
        let blk = &mut body.blocks[bi];
        blk.stmts.push((Stmt::Assign(dest, Rvalue::Use(Operand::Move(Place::local(src)))), span));
        blk.term = Term::Goto(next);
        blk.term_span = span;
    }
}

fn is_std_copy(prog: &Program, id: DefId) -> bool {
    let d = prog.def(id);
    (d.name == "clone" || d.name == "from") && prog.modules[d.module as usize].is_std
}

fn sig_matches(prog: &Program, id: DefId, targs: &[Ty], src: &Ty) -> bool {
    let f = prog.fn_def(id);
    let ret = f.ret.subst(targs);
    let Some(p0) = f.params.first() else { return false };
    let Ty::Ref { mutable: false, inner, .. } = p0.1.subst(targs) else { return false };
    *inner == ret && *src == ret
}

fn whole_local(op: &Operand) -> Option<LocalIdx> {
    match op {
        Operand::Move(p) | Operand::Copy(p) if p.is_local() => Some(p.local),
        _ => None,
    }
}

/// The owned local a reference temp points at, plus the statement that created the loan.
fn owned_ref(body: &Body, mut local: LocalIdx) -> Option<(LocalIdx, Point)> {
    for _ in 0..4 {
        let (at, rv) = single_def(body, local)?;
        match rv {
            Rvalue::Ref { mutable: false, two_phase: false, place, .. } if place.is_local() => {
                return Some((place.local, at));
            }
            Rvalue::Use(Operand::Move(p) | Operand::Copy(p)) if p.is_local() => local = p.local,
            _ => return None,
        }
    }
    None
}

fn single_def(body: &Body, local: LocalIdx) -> Option<(Point, Rvalue)> {
    let mut found: Option<(Point, Rvalue)> = None;
    for (bi, blk) in body.blocks.iter().enumerate() {
        for (si, (s, _)) in blk.stmts.iter().enumerate() {
            let Stmt::Assign(dest, rv) = s else { continue };
            if dest.local != local || !dest.is_local() {
                continue;
            }
            if found.is_some() {
                return None;
            }
            found = Some((Point { bb: bi as BlockId, idx: si }, rv.clone()));
        }
    }
    found
}

fn loan_index(loans: &[Loan], at: Point) -> Option<usize> {
    loans.iter().position(|l| l.point == at)
}

fn used_later(body: &Body, from: usize, src: LocalIdx) -> bool {
    let mut seen = vec![false; body.blocks.len()];
    let mut stack = body.successors(from as BlockId);
    while let Some(bb) = stack.pop() {
        let bb = bb as usize;
        if seen[bb] {
            continue;
        }
        seen[bb] = true;
        let blk = &body.blocks[bb];
        for (s, sp) in &blk.stmts {
            if stmt_accesses(s, *sp).iter().any(|a| touches(a, src)) {
                return true;
            }
        }
        if term_accesses(&blk.term, blk.term_span).iter().any(|a| touches(a, src)) {
            return true;
        }
        stack.extend(body.successors(bb as BlockId));
    }
    false
}

fn touches(a: &crate::borrow::Access, src: LocalIdx) -> bool {
    if a.place.local != src {
        return false;
    }
    match a.kind {
        AccessKind::Drop | AccessKind::StorageDead => false,
        AccessKind::Write if a.place.is_local() => false,
        _ => true,
    }
}

// ----- inline traces -----

struct Region {
    slot: u32,
    name: String,
    push_bb: usize,
    push_idx: usize,
    pop_bb: usize,
    pop_idx: usize,
    sink: bool,
    cover: Vec<bool>,
}

fn elide_traces(body: &mut Body) {
    let regions = regions(body);
    let n = body.blocks.len();
    let mut kill: Vec<Vec<usize>> = vec![Vec::new(); n];
    for r in &regions {
        if !r.sink {
            continue;
        }
        kill[r.push_bb].push(r.push_idx);
        kill[r.pop_bb].push(r.pop_idx);
    }
    for (bb, idxs) in kill.iter().enumerate() {
        let mut xs = idxs.clone();
        xs.sort_unstable_by(|a, b| b.cmp(a));
        xs.dedup();
        for i in xs {
            body.blocks[bb].stmts.remove(i);
        }
    }
    let mut calls: Vec<usize> = Vec::new();
    for bb in 0..n {
        if !matches!(body.blocks[bb].term, Term::Call { .. }) {
            continue;
        }
        if regions.iter().any(|r| r.sink && r.cover.get(bb).copied().unwrap_or(false)) {
            calls.push(bb);
        }
    }
    for bb in calls {
        let (next, span) = match &body.blocks[bb].term {
            Term::Call { next, span, .. } => (*next, *span),
            _ => continue,
        };
        let mut covering: Vec<usize> = regions
            .iter()
            .enumerate()
            .filter(|(_, r)| r.sink && r.cover.get(bb).copied().unwrap_or(false))
            .map(|(i, _)| i)
            .collect();
        covering.sort_by_key(|i| regions[*i].slot);
        for i in &covering {
            let r = &regions[*i];
            body.blocks[bb].stmts.push((Stmt::TracePush { name: r.name.clone(), slot: r.slot }, span));
        }
        // Outer frame first, so the call enters the inner pop and then the outer one.
        let mut dest = next;
        for i in &covering {
            dest = push_pop(body, regions[*i].slot, dest, span);
        }
        if let Term::Call { next, .. } = &mut body.blocks[bb].term {
            *next = dest;
        }
    }
}

fn push_pop(body: &mut Body, slot: u32, next: BlockId, span: crate::diag::Span) -> BlockId {
    let id = body.blocks.len() as BlockId;
    body.blocks.push(BasicBlock {
        stmts: vec![(Stmt::TracePop { slot }, span)],
        term: Term::Goto(next),
        term_span: span,
    });
    id
}

fn regions(body: &Body) -> Vec<Region> {
    let mut pushes: Vec<(u32, usize, usize, String)> = Vec::new();
    let mut pops: Vec<(u32, usize, usize)> = Vec::new();
    for (bi, blk) in body.blocks.iter().enumerate() {
        for (si, (s, _)) in blk.stmts.iter().enumerate() {
            match s {
                Stmt::TracePush { name, slot } => pushes.push((*slot, bi, si, name.clone())),
                Stmt::TracePop { slot } => pops.push((*slot, bi, si)),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    for (slot, push_bb, push_idx, name) in pushes {
        let Some((_, pop_bb, pop_idx)) = pops.iter().find(|p| p.0 == slot).copied() else { continue };
        if pops.iter().filter(|p| p.0 == slot).count() != 1 {
            continue;
        }
        let sink = push_bb != pop_bb && !cyclic(body, push_bb, pop_bb);
        let cover = if sink { cover(body, push_bb, pop_bb) } else { Vec::new() };
        out.push(Region { slot, name, push_bb, push_idx, pop_bb, pop_idx, sink, cover });
    }
    out
}

fn cover(body: &Body, start: usize, stop: usize) -> Vec<bool> {
    let mut seen = vec![false; body.blocks.len()];
    let mut stack = vec![start];
    seen[start] = true;
    while let Some(u) = stack.pop() {
        for s in body.successors(u as BlockId) {
            let s = s as usize;
            if s == stop || seen[s] {
                continue;
            }
            seen[s] = true;
            stack.push(s);
        }
    }
    seen
}

fn cyclic(body: &Body, start: usize, stop: usize) -> bool {
    let mut color = vec![0u8; body.blocks.len()];
    fn walk(body: &Body, u: usize, stop: usize, color: &mut [u8]) -> bool {
        color[u] = 1;
        for s in body.successors(u as BlockId) {
            let s = s as usize;
            if s == stop {
                continue;
            }
            if color[s] == 1 || (color[s] == 0 && walk(body, s, stop, color)) {
                return true;
            }
        }
        color[u] = 2;
        false
    }
    walk(body, start, stop, &mut color)
}

// ----- dead drops -----

fn elide_drops(body: &mut Body) {
    if !body.blocks.iter().any(|b| b.stmts.iter().any(|(s, _)| matches!(s, Stmt::Drop(p) if p.is_local()))) {
        return;
    }
    let n = body.locals.len();
    let nb = body.blocks.len();
    let mut inn: Vec<Live> = (0..nb).map(|_| Live::new(n)).collect();
    for i in 1..=body.arg_count {
        inn[0].set(i as u32);
    }
    let mut changed = true;
    while changed {
        changed = false;
        for bb in 0..nb {
            let mut st = inn[bb].clone();
            apply_block(body, bb, &mut st, false);
            for s in body.successors(bb as BlockId) {
                if inn[s as usize].or_with(&st) {
                    changed = true;
                }
            }
        }
    }
    for bb in 0..nb {
        let mut st = inn[bb].clone();
        apply_block(body, bb, &mut st, true);
    }
}

fn apply_block(body: &mut Body, bb: usize, st: &mut Live, edit: bool) {
    let nstmt = body.blocks[bb].stmts.len();
    for si in 0..nstmt {
        if edit {
            let nop = matches!(&body.blocks[bb].stmts[si].0, Stmt::Drop(p) if p.is_local() && !st.has(p.local));
            if nop {
                body.blocks[bb].stmts[si].0 = Stmt::Nop;
            }
        }
        step_stmt(&body.blocks[bb].stmts[si].0, st);
    }
    step_term(&body.blocks[bb].term, st);
}

fn step_stmt(s: &Stmt, st: &mut Live) {
    match s {
        Stmt::Assign(dest, rv) => {
            kill_rv(rv, st);
            if dest.is_local() {
                st.set(dest.local);
            }
        }
        Stmt::Drop(p) if p.is_local() => st.clear(p.local),
        Stmt::Forget(l) => st.clear(*l),
        _ => {}
    }
}

fn step_term(t: &Term, st: &mut Live) {
    match t {
        Term::If(op, ..) | Term::Switch(op, ..) => kill_op(op, st),
        Term::Call { func, args, dest, .. } => {
            if let Func::Ptr(op) | Func::Clos(op) = func {
                kill_op(op, st);
            }
            for a in args {
                kill_op(a, st);
            }
            if dest.is_local() {
                st.set(dest.local);
            }
        }
        _ => {}
    }
}

fn kill_rv(rv: &Rvalue, st: &mut Live) {
    match rv {
        Rvalue::Use(op) | Rvalue::Cast(op, _) | Rvalue::Neg(op) | Rvalue::WNeg(op) | Rvalue::BNot(op) | Rvalue::Not(op) | Rvalue::BoxPack { value: op, .. } => {
            kill_op(op, st);
        }
        Rvalue::Bin(_, a, b) | Rvalue::PtrAdd(a, b, _) | Rvalue::PtrEq(a, b) => {
            kill_op(a, st);
            kill_op(b, st);
        }
        Rvalue::Struct(_, xs) | Rvalue::Variant(_, _, xs) | Rvalue::Closure { caps: xs, .. } => {
            for o in xs {
                kill_op(o, st);
            }
        }
        _ => {}
    }
}

/// A move that clears the whole-local drop flag, matching `codegen/func.rs`.
fn kill_op(op: &Operand, st: &mut Live) {
    let Operand::Move(p) = op else { return };
    if p.has_deref() || p.lone_field().is_some() || p.proj.iter().any(|x| matches!(x, Proj::Index(_))) {
        return;
    }
    st.clear(p.local);
}

#[derive(Clone)]
struct Live(Vec<u64>);

impl Live {
    fn new(n: usize) -> Live {
        Live(vec![0; (n + 63) / 64])
    }
    fn set(&mut self, i: LocalIdx) {
        let i = i as usize;
        self.0[i / 64] |= 1u64 << (i % 64);
    }
    fn clear(&mut self, i: LocalIdx) {
        let i = i as usize;
        if i / 64 < self.0.len() {
            self.0[i / 64] &= !(1u64 << (i % 64));
        }
    }
    fn has(&self, i: LocalIdx) -> bool {
        let i = i as usize;
        self.0.get(i / 64).map(|w| w & (1u64 << (i % 64)) != 0).unwrap_or(false)
    }
    fn or_with(&mut self, o: &Live) -> bool {
        let mut changed = false;
        for (a, b) in self.0.iter_mut().zip(&o.0) {
            let n = *a | *b;
            changed |= n != *a;
            *a = n;
        }
        changed
    }
}
