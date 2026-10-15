//! Match lowering: decision chain over arms, bindings by move or by reference.
use super::lower::Lower;
use super::*;
use crate::ast::Lit;
use crate::thir::{Arm, Expr, Pat};

impl<'a> Lower<'a> {
    pub fn lower_match(&mut self, scrut: &Expr, arms: &[Arm], by_ref: Option<bool>, dest: Place, sp: Span) {
        // Scrutinee place: through the reference when matching on `(& enum)`.
        let base = match by_ref {
            Some(_) => {
                let op = self.as_operand(scrut);
                let t = self.temp(scrut.ty.clone(), scrut.span);
                self.assign(&Place::local(t), Rvalue::Use(op), scrut.span);
                Place::local(t).deref()
            }
            None => self.as_place(scrut),
        };
        let join = self.new_block();
        for arm in arms {
            let fail = self.new_block();
            let mut binds: Vec<(LocalIdx, Place)> = Vec::new();
            let mut drops: Vec<Place> = Vec::new();
            self.test_pat(&arm.pat, base.clone(), fail, &mut binds, &mut drops, arm.span);
            // Bindings run after all tests pass.
            self.push_scope();
            for (l, p) in binds {
                let pty = place_ty(self.prog, &self.body, &p);
                let rv = match by_ref {
                    Some(m) => {
                        self.scopes.last_mut().unwrap().all.push(l);
                        Rvalue::Ref { mutable: m, place: p, two_phase: false }
                    }
                    None => {
                        let sc = self.scopes.last_mut().unwrap();
                        sc.all.push(l);
                        if needs_drop(self.prog, &pty) {
                            sc.drops.push(l);
                        }
                        let op = if self.is_copy(&pty) { Operand::Copy(p) } else { Operand::Move(p) };
                        Rvalue::Use(op)
                    }
                };
                self.assign(&Place::local(l), rv, arm.span);
            }
            if by_ref.is_none() {
                for d in drops {
                    self.push(Stmt::Drop(d), arm.span);
                }
            }
            self.expr_into(&arm.body, dest.clone(), sp);
            self.pop_scope(arm.span);
            self.terminate(Term::Goto(join), sp);
            self.cur = fail;
        }
        self.terminate(Term::Unreachable, sp);
        self.cur = join;
    }

    /// Emits tests for `pat` against `place` in the current block, jumping to `fail` on
    /// mismatch. Continues in a new block on success. Collects bindings and wildcard
    /// payload places that must be dropped when matching by value.
    fn test_pat(&mut self, pat: &Pat, place: Place, fail: BlockId, binds: &mut Vec<(LocalIdx, Place)>, drops: &mut Vec<Place>, sp: Span) {
        match pat {
            Pat::Wild => {
                let pty = place_ty(self.prog, &self.body, &place);
                // Only payload fields (not the whole scrutinee) are dropped here.
                if place.proj.iter().any(|p| matches!(p, Proj::Downcast(_))) && needs_drop(self.prog, &pty) {
                    drops.push(place);
                }
            }
            Pat::Bind(l) => {
                let idx = self.local_map[l.0 as usize];
                binds.push((idx, place));
            }
            Pat::Lit(l) => {
                let pty = place_ty(self.prog, &self.body, &place);
                let c = super::lower::lit_const(l, &pty);
                let ok = self.new_block();
                match (l, &c) {
                    (Lit::Bool(b), _) => {
                        let (t, f) = if *b { (ok, fail) } else { (fail, ok) };
                        self.terminate(Term::If(Operand::Copy(place), t, f), sp);
                    }
                    (_, Const::Int(v, _)) => {
                        self.terminate(Term::Switch(Operand::Copy(place), vec![(*v as u128, ok)], fail), sp);
                    }
                    _ => {
                        self.terminate(Term::Goto(fail), sp);
                    }
                }
                self.cur = ok;
            }
            Pat::Variant { idx, subs } => {
                let d = self.temp(Ty::Int(IntTy::U32), sp);
                self.assign(&Place::local(d), Rvalue::Discr(place.clone()), sp);
                let ok = self.new_block();
                self.terminate(Term::Switch(Operand::Copy(Place::local(d)), vec![(*idx as u128, ok)], fail), sp);
                self.cur = ok;
                for (i, s) in subs.iter().enumerate() {
                    let fp = place.clone().downcast(*idx).field(i as u32);
                    self.test_pat(s, fp, fail, binds, drops, sp);
                }
            }
        }
    }
}
