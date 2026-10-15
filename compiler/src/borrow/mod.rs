//! Borrow checker over MIR: moves, loans with liveness-based (NLL) regions, conflicts.
mod check;
mod holders;
mod liveness;
mod loans;
mod moves;

use crate::diag::{Diags, SourceMap, Span};
use crate::mir::*;
use crate::resolve::Program;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point {
    pub bb: BlockId,
    pub idx: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessKind {
    Read,
    Write,
    Move,
    BorrowShared,
    BorrowMut,
    Drop,
    StorageDead,
}

#[derive(Clone, Debug)]
pub struct Access {
    pub place: Place,
    pub kind: AccessKind,
    pub span: Span,
}

/// Compact bitset over locals.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BitSet(Vec<u64>);

impl BitSet {
    pub fn new(n: usize) -> BitSet {
        BitSet(vec![0; (n + 63) / 64])
    }
    pub fn insert(&mut self, i: u32) -> bool {
        let w = &mut self.0[i as usize / 64];
        let m = 1u64 << (i % 64);
        let was = *w & m != 0;
        *w |= m;
        !was
    }
    pub fn remove(&mut self, i: u32) {
        self.0[i as usize / 64] &= !(1u64 << (i % 64));
    }
    pub fn contains(&self, i: u32) -> bool {
        self.0[i as usize / 64] & (1u64 << (i % 64)) != 0
    }
    pub fn union_with(&mut self, o: &BitSet) -> bool {
        let mut changed = false;
        for (a, b) in self.0.iter_mut().zip(&o.0) {
            let n = *a | *b;
            changed |= n != *a;
            *a = n;
        }
        changed
    }
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.0.iter().enumerate().flat_map(|(wi, w)| (0..64).filter(move |b| w & (1u64 << b) != 0).map(move |b| (wi * 64 + b) as u32))
    }
}

fn operand_access(op: &Operand, span: Span, out: &mut Vec<Access>) {
    match op {
        Operand::Copy(p) => out.push(Access { place: p.clone(), kind: AccessKind::Read, span }),
        Operand::Move(p) => out.push(Access { place: p.clone(), kind: AccessKind::Move, span }),
        Operand::Const(..) => {}
    }
}

/// Accesses performed by a statement, in evaluation order (reads before the write).
pub fn stmt_accesses(s: &Stmt, span: Span) -> Vec<Access> {
    let mut out = Vec::new();
    match s {
        Stmt::Assign(dest, rv) => {
            match rv {
                Rvalue::Use(op) | Rvalue::Neg(op) | Rvalue::Not(op) | Rvalue::Cast(op, _) => operand_access(op, span, &mut out),
                Rvalue::Ref { mutable, place, .. } => {
                    let kind = if *mutable { AccessKind::BorrowMut } else { AccessKind::BorrowShared };
                    out.push(Access { place: place.clone(), kind, span });
                }
                Rvalue::Bin(_, a, b) | Rvalue::PtrAdd(a, b, _) | Rvalue::PtrEq(a, b) => {
                    operand_access(a, span, &mut out);
                    operand_access(b, span, &mut out);
                }
                Rvalue::Struct(_, ops) | Rvalue::Variant(_, _, ops) => ops.iter().for_each(|o| operand_access(o, span, &mut out)),
                Rvalue::Discr(p) => out.push(Access { place: p.clone(), kind: AccessKind::Read, span }),
                Rvalue::PtrNull | Rvalue::Argc | Rvalue::Argv | Rvalue::Sizeof(_) => {}
            }
            out.push(Access { place: dest.clone(), kind: AccessKind::Write, span });
        }
        Stmt::Drop(p) => out.push(Access { place: p.clone(), kind: AccessKind::Drop, span }),
        Stmt::Forget(l) => out.push(Access { place: Place::local(*l), kind: AccessKind::Move, span }),
        Stmt::StorageDead(l) => out.push(Access { place: Place::local(*l), kind: AccessKind::StorageDead, span }),
        Stmt::Nop => {}
    }
    out
}

pub fn term_accesses(t: &Term, span: Span) -> Vec<Access> {
    let mut out = Vec::new();
    match t {
        Term::If(op, ..) | Term::Switch(op, ..) => operand_access(op, span, &mut out),
        Term::Call { func, args, dest, .. } => {
            if let Func::Ptr(op) = func {
                operand_access(op, span, &mut out);
            }
            args.iter().for_each(|a| operand_access(a, span, &mut out));
            out.push(Access { place: dest.clone(), kind: AccessKind::Write, span });
        }
        Term::Return => out.push(Access { place: Place::local(0), kind: AccessKind::Read, span }),
        Term::Goto(_) | Term::Unreachable | Term::None => {}
    }
    out
}

/// Two places overlap when they share a root and one projection path prefixes the other.
pub fn overlaps(a: &Place, b: &Place) -> bool {
    if a.local != b.local {
        return false;
    }
    let pa: Vec<&Proj> = a.proj.iter().filter(|p| !matches!(p, Proj::Downcast(_))).collect();
    let pb: Vec<&Proj> = b.proj.iter().filter(|p| !matches!(p, Proj::Downcast(_))).collect();
    pa.iter().zip(&pb).all(|(x, y)| x == y)
}

pub fn borrowck(prog: &Program, diags: &Diags, sm: &SourceMap, body: &Body) {
    let live = liveness::compute(prog, body);
    moves::check_moves(prog, diags, sm, body);
    let loans = loans::compute(body);
    let holders = holders::compute(prog, body, &loans);
    check::check_conflicts(prog, diags, sm, body, &loans, &holders, &live);
}
