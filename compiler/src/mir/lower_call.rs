//! Lowering of calls, builtins and `try`.
use super::lower::Lower;
use super::*;
use crate::thir::{Builtin, Callee, Expr, ExprKind};

impl<'a> Lower<'a> {
    pub fn lower_call(&mut self, callee: &Callee, args: &[Expr], ret: &Ty, dest: Place, sp: Span) {
        match callee {
            Callee::Builtin(b) => return self.lower_builtin(*b, args, dest, sp),
            Callee::Sizeof(t) => return self.assign(&dest, Rvalue::Sizeof(t.clone()), sp),
            _ => {}
        }
        let mut ops = Vec::new();
        for a in args {
            // two-phase &mut borrows directly in argument position
            if let ExprKind::Ref { mutable: true, place } = &a.kind {
                if place.is_place() {
                    let p = self.as_place(place);
                    let t = self.temp(a.ty.clone(), a.span);
                    self.assign(&Place::local(t), Rvalue::Ref { mutable: true, place: p, two_phase: true }, a.span);
                    ops.push(Operand::Move(Place::local(t)));
                    continue;
                }
            }
            if a.is_place() && matches!(a.ty, Ty::Ref { mutable: true, .. }) {
                let p = self.as_place(a);
                ops.push(self.reborrow(p, &a.ty, true, a.span));
                continue;
            }
            ops.push(self.as_operand(a));
        }
        let func = match callee {
            Callee::Fn(d, targs) => Func::Def(*d, targs.clone()),
            Callee::Ptr(e) => Func::Ptr(self.as_operand(e)),
            _ => unreachable!(),
        };
        let next = self.new_block();
        self.terminate(Term::Call { func, args: ops, dest, next, span: sp }, sp);
        self.cur = next;
        if matches!(ret, Ty::Never) {
            self.terminate(Term::Unreachable, sp);
            self.cur = self.new_block();
        }
    }

    fn lower_builtin(&mut self, b: Builtin, args: &[Expr], dest: Place, sp: Span) {
        match b {
            Builtin::PtrNull => self.assign(&dest, Rvalue::PtrNull, sp),
            Builtin::PtrAdd => {
                let p = self.as_operand(&args[0]);
                let n = self.as_operand(&args[1]);
                let elem = match &args[0].ty {
                    Ty::Ptr(i) => (**i).clone(),
                    _ => unreachable!(),
                };
                self.assign(&dest, Rvalue::PtrAdd(p, n, elem), sp);
            }
            Builtin::PtrRead => {
                let p = self.deref_via_temp(&args[0]).unwrap();
                self.assign(&dest, Rvalue::Use(Operand::Copy(p)), sp);
            }
            Builtin::PtrWrite => {
                let v = self.as_operand(&args[1]);
                let p = self.deref_via_temp(&args[0]).unwrap();
                self.assign(&p, Rvalue::Use(v), sp);
            }
            Builtin::PtrEq => {
                let a = self.as_operand(&args[0]);
                let b2 = self.as_operand(&args[1]);
                self.assign(&dest, Rvalue::PtrEq(a, b2), sp);
            }
            Builtin::MemReplace => {
                let v = self.as_operand(&args[1]);
                let p = self.deref_via_temp(&args[0]).unwrap();
                self.assign(&dest, Rvalue::Use(Operand::Copy(p.clone())), sp);
                self.assign(&p, Rvalue::Use(v), sp);
            }
            Builtin::Forget => {
                let op = self.as_operand(&args[0]);
                if let Some(pl) = op.place() {
                    if pl.is_local() {
                        self.push(Stmt::Forget(pl.local), sp);
                        return;
                    }
                }
                let t = self.temp(args[0].ty.clone(), sp);
                self.assign(&Place::local(t), Rvalue::Use(op), sp);
                self.push(Stmt::Forget(t), sp);
            }
            Builtin::Drop => {
                let op = self.as_operand(&args[0]);
                let t = self.temp(args[0].ty.clone(), sp);
                self.assign(&Place::local(t), Rvalue::Use(op), sp);
                self.push(Stmt::Drop(Place::local(t)), sp);
            }
            Builtin::Argc => self.assign(&dest, Rvalue::Argc, sp),
            Builtin::Argv => self.assign(&dest, Rvalue::Argv, sp),
        }
    }

    /// Evaluates a pointer/reference expression into a temp and returns `*temp`.
    fn deref_via_temp(&mut self, e: &Expr) -> Option<Place> {
        let op = self.as_operand(e);
        let t = self.temp(e.ty.clone(), e.span);
        self.assign(&Place::local(t), Rvalue::Use(op), e.span);
        Some(Place::local(t).deref())
    }

    pub fn lower_try(&mut self, inner: &Expr, is_res: bool, dest: Place, sp: Span) {
        let t = self.temp(inner.ty.clone(), sp);
        self.expr_into(inner, Place::local(t), sp);
        let d = self.temp(Ty::Int(IntTy::U32), sp);
        self.assign(&Place::local(d), Rvalue::Discr(Place::local(t)), sp);
        let ok = self.new_block();
        let err = self.new_block();
        self.terminate(Term::Switch(Operand::Copy(Place::local(d)), vec![(0, ok)], err), sp);
        self.cur = err;
        let ret_ty = self.ret_ty.clone();
        if is_res {
            let ep = Place::local(t).downcast(1).field(0);
            let ety = place_ty(self.prog, &self.body, &ep);
            let eop = if self.is_copy(&ety) { Operand::Copy(ep) } else { Operand::Move(ep) };
            self.assign(&Place::local(0), Rvalue::Variant(ret_ty, 1, vec![eop]), sp);
        } else {
            self.assign(&Place::local(0), Rvalue::Variant(ret_ty, 1, Vec::new()), sp);
        }
        self.drops_to_depth(0, sp);
        self.terminate(Term::Return, sp);
        self.cur = ok;
        let vp = Place::local(t).downcast(0).field(0);
        let vty = place_ty(self.prog, &self.body, &vp);
        let vop = if self.is_copy(&vty) { Operand::Copy(vp) } else { Operand::Move(vp) };
        if !vty.is_unit() {
            self.assign(&dest, Rvalue::Use(vop), sp);
        }
    }
}
