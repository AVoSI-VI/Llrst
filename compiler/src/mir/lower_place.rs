//! Places and operands. An index projection is only built from a local.
use super::lower::Lower;
use super::*;
use crate::diag::Diag;
use crate::thir::{Expr, ExprKind};

impl<'a> Lower<'a> {
    pub fn as_place(&mut self, e: &Expr) -> Place {
        match &e.kind {
            ExprKind::Local(l) => Place::local(self.local(*l)),
            ExprKind::Field { base, idx } => self.as_place(base).field(*idx),
            ExprKind::Index { base, idx } => {
                let ExprKind::Local(id) = &idx.kind else { panic!("index local") };
                self.as_place(base).index(self.local(*id))
            }
            ExprKind::Deref(inner) => {
                if let ExprKind::Local(l) = &inner.kind {
                    return Place::local(self.local(*l)).deref();
                }
                let op = self.as_operand(inner);
                let t = self.temp(inner.ty.clone(), inner.span);
                self.push(Stmt::Assign(Place::local(t), Rvalue::Use(op)), inner.span);
                Place::local(t).deref()
            }
            _ => {
                let t = self.temp(e.ty.clone(), e.span);
                self.expr_into(e, Place::local(t), e.span);
                Place::local(t)
            }
        }
    }

    pub fn as_operand(&mut self, e: &Expr) -> Operand {
        match &e.kind {
            ExprKind::Lit(l) => Operand::Const(super::lower::lit_const(l, &e.ty), e.ty.clone()),
            ExprKind::FnRef(d) => Operand::Const(Const::Fn(*d, Vec::new()), e.ty.clone()),
            ExprKind::Index { .. } => panic!("index is only a place"),
            ExprKind::Local(_) | ExprKind::Field { .. } | ExprKind::Deref(_) => {
                let p = self.as_place(e);
                if matches!(e.ty, Ty::Ref { mutable: true, .. }) {
                    return self.reborrow(p, &e.ty, false, e.span);
                }
                if self.is_copy(&e.ty) {
                    Operand::Copy(p)
                } else {
                    if !p.is_local() && !self.can_move_field(&p) {
                        let code = if p.has_deref() { "move.deref" } else { "move.field" };
                        self.diags.push(Diag::error(e.span, code).kv("fix", "(& ...) or mem.replace"));
                    }
                    Operand::Move(p)
                }
            }
            _ => {
                if e.ty.is_unit() {
                    self.expr_stmt(e);
                    return Operand::Const(Const::Unit, Ty::Unit);
                }
                let t = self.temp(e.ty.clone(), e.span);
                self.expr_into(e, Place::local(t), e.span);
                Operand::Move(Place::local(t))
            }
        }
    }

    /// Implicit reborrow: using a `&mut` place yields `&mut *place` instead of a move.
    pub fn reborrow(&mut self, p: Place, ty: &Ty, two_phase: bool, span: crate::diag::Span) -> Operand {
        let t = self.temp(ty.clone(), span);
        self.assign(&Place::local(t), Rvalue::Ref { mutable: true, place: p.deref(), two_phase }, span);
        Operand::Move(Place::local(t))
    }

    /// One field of a struct local that has no user `drop` method.
    pub(crate) fn can_move_field(&self, p: &Place) -> bool {
        if p.lone_field().is_none() {
            return false;
        }
        let ty = self.body.local_ty(p.local);
        if drop_method(self.prog, ty).is_some() {
            return false;
        }
        let Ty::Adt(d, _) = ty else { return false };
        matches!(self.prog.def(*d).kind, crate::resolve::DefKind::Struct(_))
    }
}
