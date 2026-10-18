//! Forward "maybe moved" dataflow and use-after-move errors.
use super::*;
use crate::diag::{Diag, SourceMap};
use std::collections::{HashMap, HashSet};

struct Moved {
    whole: BitSet,
    fields: HashSet<(LocalIdx, u32)>,
}

impl Moved {
    fn new(n: usize) -> Moved {
        Moved { whole: BitSet::new(n), fields: HashSet::new() }
    }
    fn clone_at(&self, n: usize) -> Moved {
        let mut whole = BitSet::new(n);
        whole.union_with(&self.whole);
        Moved { whole, fields: self.fields.clone() }
    }
    fn union_with(&mut self, o: &Moved) -> bool {
        let w = self.whole.union_with(&o.whole);
        let before = self.fields.len();
        self.fields.extend(o.fields.iter().copied());
        w || self.fields.len() != before
    }
}

pub fn check_moves(prog: &Program, diags: &Diags, sm: &SourceMap, body: &Body) {
    let _ = prog;
    let n = body.locals.len();
    let nb = body.blocks.len();
    let mut moved_in: Vec<Moved> = (0..nb).map(|_| Moved::new(n)).collect();
    let mut moved_at: HashMap<LocalIdx, Span> = HashMap::new();
    let mut field_at: HashMap<(LocalIdx, u32), Span> = HashMap::new();
    let mut changed = true;
    while changed {
        changed = false;
        for bb in 0..nb {
            let mut state = moved_in[bb].clone_at(n);
            transfer(body, bb as BlockId, &mut state, &mut moved_at, &mut field_at, None);
            for s in body.successors(bb as BlockId) {
                if moved_in[s as usize].union_with(&state) {
                    changed = true;
                }
            }
        }
    }
    let mut reported: Vec<(LocalIdx, Span)> = Vec::new();
    for bb in 0..nb {
        let mut state = moved_in[bb].clone_at(n);
        let mut errs = Vec::new();
        transfer(body, bb as BlockId, &mut state, &mut moved_at, &mut field_at, Some(&mut errs));
        for (l, sp, at) in errs {
            if reported.contains(&(l, sp)) {
                continue;
            }
            reported.push((l, sp));
            let name = body.locals[l as usize].name.clone();
            let mut d = Diag::error(sp, "move.used").kv("name", name);
            if let Some(m) = at {
                let (line, col) = sm.line_col(m);
                d = d.kv("moved_at", format!("{}:{}", line, col));
            }
            diags.push(d);
        }
    }
}

fn transfer(
    body: &Body,
    bb: BlockId,
    state: &mut Moved,
    moved_at: &mut HashMap<LocalIdx, Span>,
    field_at: &mut HashMap<(LocalIdx, u32), Span>,
    mut errs: Option<&mut Vec<(LocalIdx, Span, Option<Span>)>>,
) {
    let blk = &body.blocks[bb as usize];
    let mut all: Vec<Vec<Access>> = blk.stmts.iter().map(|(s, sp)| stmt_accesses(s, *sp)).collect();
    all.push(term_accesses(&blk.term, blk.term_span));
    for accs in all {
        for a in &accs {
            if !uses_moved(a.kind) && !(a.kind == AccessKind::Write && !a.place.is_local()) {
                continue;
            }
            if let Some(at) = moved_span(state, moved_at, field_at, &a.place) {
                if let Some(e) = errs.as_mut() {
                    e.push((a.place.local, a.span, at));
                }
            }
        }
        for a in &accs {
            let l = a.place.local;
            match a.kind {
                AccessKind::Move if !a.place.has_deref() => {
                    if let Some(i) = a.place.lone_field() {
                        state.fields.insert((l, i));
                        field_at.entry((l, i)).or_insert(a.span);
                    } else {
                        state.whole.insert(l);
                        moved_at.entry(l).or_insert(a.span);
                    }
                }
                AccessKind::Write if a.place.is_local() => {
                    state.whole.remove(l);
                    state.fields.retain(|(ll, _)| *ll != l);
                }
                _ => {}
            }
        }
    }
}

fn uses_moved(k: AccessKind) -> bool {
    matches!(k, AccessKind::Read | AccessKind::Move | AccessKind::BorrowShared | AccessKind::BorrowMut)
}

fn moved_span(state: &Moved, moved_at: &HashMap<LocalIdx, Span>, field_at: &HashMap<(LocalIdx, u32), Span>, place: &Place) -> Option<Option<Span>> {
    let l = place.local;
    if state.whole.contains(l) {
        return Some(moved_at.get(&l).copied());
    }
    if let Some(i) = place.lone_field() {
        if state.fields.contains(&(l, i)) {
            return Some(field_at.get(&(l, i)).copied());
        }
        return None;
    }
    if place.is_local() && state.fields.iter().any(|(ll, _)| *ll == l) {
        let at = state.fields.iter().find(|(ll, _)| *ll == l).and_then(|(ll, i)| field_at.get(&(*ll, *i)).copied());
        return Some(at);
    }
    None
}
