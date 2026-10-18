//! Warnings over a lowered body. They do not fail the build.
use super::*;
use crate::diag::{Diag, Diags};

pub fn lint(diags: &Diags, body: &Body) {
    warn_unreachable(diags, body);
    warn_unused(diags, body);
}

fn warn_unreachable(diags: &Diags, body: &Body) {
    let n = body.blocks.len();
    let mut seen = vec![false; n];
    let mut stack = vec![0u32];
    if n > 0 {
        seen[0] = true;
    }
    while let Some(b) = stack.pop() {
        for s in body.successors(b) {
            if !seen[s as usize] {
                seen[s as usize] = true;
                stack.push(s);
            }
        }
    }
    for (i, reach) in seen.iter().enumerate() {
        if *reach {
            continue;
        }
        let blk = &body.blocks[i];
        let interesting = blk.stmts.iter().find(|(s, _)| !is_cleanup(s));
        if let Some((_, span)) = interesting {
            diags.push(Diag::warn(*span, "flow.unreachable"));
        }
    }
}

fn is_cleanup(s: &Stmt) -> bool {
    matches!(s, Stmt::Drop(_) | Stmt::StorageDead(_) | Stmt::Nop | Stmt::InlineLoans { .. })
}

fn warn_unused(diags: &Diags, body: &Body) {
    let mut read = vec![false; body.locals.len()];
    for blk in &body.blocks {
        for (s, _) in &blk.stmts {
            mark_stmt(&mut read, s);
        }
        mark_term(&mut read, &blk.term);
    }
    for (i, loc) in body.locals.iter().enumerate().skip(1) {
        if loc.name.starts_with('_') || read[i] {
            continue;
        }
        diags.push(Diag::warn(loc.span, "name.unused").kv("name", loc.name.clone()));
    }
}

fn mark_place(read: &mut [bool], p: &Place) {
    read[p.local as usize] = true;
}

fn mark_op(read: &mut [bool], op: &Operand) {
    if let Some(p) = op.place() {
        mark_place(read, p);
    }
}

fn mark_stmt(read: &mut [bool], s: &Stmt) {
    let Stmt::Assign(_, rv) = s else { return };
    match rv {
        Rvalue::Use(op) | Rvalue::Neg(op) | Rvalue::WNeg(op) | Rvalue::Not(op) | Rvalue::Cast(op, _) => mark_op(read, op),
        Rvalue::Ref { place, .. } | Rvalue::Discr(place) => mark_place(read, place),
        Rvalue::Bin(_, a, b) | Rvalue::PtrAdd(a, b, _) | Rvalue::PtrEq(a, b) => {
            mark_op(read, a);
            mark_op(read, b);
        }
        Rvalue::Struct(_, ops) | Rvalue::Variant(_, _, ops) => ops.iter().for_each(|o| mark_op(read, o)),
        Rvalue::PtrNull | Rvalue::Argc | Rvalue::Argv | Rvalue::Sizeof(_) => {}
    }
}

fn mark_term(read: &mut [bool], t: &Term) {
    match t {
        Term::If(op, ..) | Term::Switch(op, ..) => mark_op(read, op),
        Term::Call { func, args, .. } => {
            if let Func::Ptr(op) = func {
                mark_op(read, op);
            }
            args.iter().for_each(|a| mark_op(read, a));
        }
        Term::Return => read[0] = true,
        Term::Goto(_) | Term::Unreachable | Term::None => {}
    }
}
