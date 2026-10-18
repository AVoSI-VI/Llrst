//! Conflict detection between live loans and accesses.
use super::holders::Holders;
use super::liveness::Liveness;
use super::loans::Loan;
use super::*;
use crate::diag::{Diag, SourceMap};

pub fn check_conflicts(prog: &Program, diags: &Diags, sm: &SourceMap, body: &Body, loans: &[Loan], holders: &Holders, live: &Liveness) {
    let _ = prog;
    let static_l = super::holders::static_locals(body);
    let mut reported: Vec<(usize, Span, &'static str)> = Vec::new();
    for (bi, blk) in body.blocks.iter().enumerate() {
        let mut all: Vec<(Vec<Access>, Point)> = blk
            .stmts
            .iter()
            .enumerate()
            .map(|(si, (s, sp))| (stmt_accesses(s, *sp), Point { bb: bi as BlockId, idx: si }))
            .collect();
        all.push((term_accesses(&blk.term, blk.term_span), Point { bb: bi as BlockId, idx: blk.stmts.len() }));
        for (accs, point) in all {
            for (li, loan) in loans.iter().enumerate() {
                if loan.point == point || static_l[loan.place.local as usize] {
                    continue;
                }
                // A loan is in force at `point` when a holder is live after the borrow and
                // still live here (the borrow point itself is excluded above).
                if !holders.in_force(point, li, live) && !holder_live_after(holders, loan, li, point, live, &accs) {
                    continue;
                }
                for a in &accs {
                    if !overlaps(&loan.place, &a.place) {
                        continue;
                    }
                    // Accessing through the reference itself (holder as root) is fine.
                    if holders.held(point, li, a.place.local) && a.place.local != loan.place.local {
                        continue;
                    }
                    let active = loan.active_at(point);
                    // A loan through a dereference does not borrow the local's own storage.
                    if matches!(a.kind, AccessKind::Drop | AccessKind::StorageDead) && loan.place.has_deref() {
                        continue;
                    }
                    let code: Option<&'static str> = match a.kind {
                        AccessKind::Read => (loan.mutable && active).then_some("borrow.mut_shared"),
                        AccessKind::BorrowShared => (loan.mutable && active).then_some("borrow.mut_shared"),
                        AccessKind::BorrowMut => Some(if loan.mutable && active { "borrow.mut_mut" } else { "borrow.mut_shared" }),
                        AccessKind::Write => Some("borrow.assign"),
                        AccessKind::Move => Some("move.borrowed"),
                        AccessKind::Drop | AccessKind::StorageDead => Some("borrow.escape"),
                    };
                    let Some(code) = code else { continue };
                    if reported.contains(&(li, a.span, code)) {
                        continue;
                    }
                    reported.push((li, a.span, code));
                    let name = body.locals[loan.place.local as usize].name.clone();
                    let holder = holders.some_holder_name(point, li, body);
                    let at = if code == "borrow.escape" { loan.span } else { a.span };
                    let mut d = Diag::error(at, code).kv("name", name);
                    if !holder.is_empty() {
                        d = d.kv("ref", holder);
                    }
                    let (line, col) = sm.line_col(loan.span);
                    d = d.kv("borrowed_at", format!("{}:{}", line, col));
                    diags.push(d);
                }
            }
        }
    }
}

/// For `Drop`/`StorageDead` of the borrowed local, the question is whether a holder is live
/// *after* this point (the referent dies while the reference lives on).
fn holder_live_after(holders: &Holders, loan: &Loan, li: usize, point: Point, live: &Liveness, accs: &[Access]) -> bool {
    if !accs.iter().any(|a| matches!(a.kind, AccessKind::Drop | AccessKind::StorageDead) && a.place.local == loan.place.local) {
        return false;
    }
    holders.any_holder_live_after(point, li, live)
}
