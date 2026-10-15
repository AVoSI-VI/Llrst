//! Forward "maybe moved" dataflow and use-after-move errors.
use super::*;
use crate::diag::{Diag, SourceMap};
use std::collections::HashMap;

pub fn check_moves(prog: &Program, diags: &Diags, sm: &SourceMap, body: &Body) {
    let _ = prog;
    let n = body.locals.len();
    let nb = body.blocks.len();
    let mut moved_in: Vec<BitSet> = (0..nb).map(|_| BitSet::new(n)).collect();
    let mut moved_at: HashMap<LocalIdx, Span> = HashMap::new();
    let mut changed = true;
    while changed {
        changed = false;
        for bb in 0..nb {
            let mut state = moved_in[bb].clone();
            transfer(body, bb as BlockId, &mut state, &mut moved_at, None);
            for s in body.successors(bb as BlockId) {
                if moved_in[s as usize].union_with(&state) {
                    changed = true;
                }
            }
        }
    }
    let mut reported: Vec<(LocalIdx, Span)> = Vec::new();
    for bb in 0..nb {
        let mut state = moved_in[bb].clone();
        let mut errs = Vec::new();
        transfer(body, bb as BlockId, &mut state, &mut moved_at, Some(&mut errs));
        for (l, sp) in errs {
            if reported.contains(&(l, sp)) {
                continue;
            }
            reported.push((l, sp));
            let name = body.locals[l as usize].name.clone();
            let mut d = Diag::error(sp, "move.used").kv("name", name);
            if let Some(m) = moved_at.get(&l) {
                let (line, col) = sm.line_col(*m);
                d = d.kv("moved_at", format!("{}:{}", line, col));
            }
            diags.push(d);
        }
    }
}

fn transfer(body: &Body, bb: BlockId, state: &mut BitSet, moved_at: &mut HashMap<LocalIdx, Span>, mut errs: Option<&mut Vec<(LocalIdx, Span)>>) {
    let blk = &body.blocks[bb as usize];
    let mut all: Vec<Vec<Access>> = blk.stmts.iter().map(|(s, sp)| stmt_accesses(s, *sp)).collect();
    all.push(term_accesses(&blk.term, blk.term_span));
    for accs in all {
        for a in &accs {
            let l = a.place.local;
            match a.kind {
                AccessKind::Read | AccessKind::Move | AccessKind::BorrowShared | AccessKind::BorrowMut => {
                    if state.contains(l) {
                        if let Some(e) = errs.as_mut() {
                            e.push((l, a.span));
                        }
                    }
                }
                AccessKind::Write if !a.place.is_local() => {
                    if state.contains(l) {
                        if let Some(e) = errs.as_mut() {
                            e.push((l, a.span));
                        }
                    }
                }
                _ => {}
            }
        }
        // Effects after checks: moves mark, whole-local writes re-initialize.
        for a in &accs {
            let l = a.place.local;
            match a.kind {
                AccessKind::Move if !a.place.has_deref() => {
                    state.insert(l);
                    moved_at.entry(l).or_insert(a.span);
                }
                AccessKind::Write if a.place.is_local() => state.remove(l),
                _ => {}
            }
        }
    }
}
