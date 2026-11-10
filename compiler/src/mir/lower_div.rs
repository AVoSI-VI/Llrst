//! Integer `/` and `%` become a call to `sys.checked_div`, which panics on a zero divisor
//! and on signed minimum divided or remaindered by `-1`.
//! `i128` and `u128` call `sys.div128` and `sys.udiv128` with no widening.
//! Under `--checked`, a 128-bit `*` calls `sys.checked_mul128` or `sys.checked_umul128`.
//! The function's own division is lowered raw (`Lower::raw_div`) so the rewrite does not recurse.
use super::lower::Lower;
use super::*;
use crate::ast::BinOp;
use crate::thir::Expr;
use crate::ty::DefId;

impl<'a> Lower<'a> {
    pub fn lower_checked_div(&mut self, op: BinOp, l: &Expr, r: &Expr, dest: &Place, ty: Ty, span: Span) {
        if matches!(ty, Ty::Int(i) if i.wide()) {
            let name = if ty == Ty::Int(IntTy::I128) { "div128" } else { "udiv128" };
            self.lower_wide_op(name, true, op, l, r, dest, ty, span);
            return;
        }
        let def = self.checked_div_id();
        let a = self.widen_i64(l);
        let b = self.widen_i64(r);
        let (line, col) = self.sm.line_col(span);
        let loc = format!("{}|{}|{}", self.sm.path(span.file), line, col);
        let loc_op = Operand::Const(Const::Str(loc.into_bytes()), self.ref_str());
        let rem = Operand::Const(Const::Bool(op == BinOp::Rem), Ty::Bool);
        let tmp = self.temp(Ty::Int(IntTy::I64), span);
        let next = self.new_block();
        self.terminate(Term::Call { func: Func::Def(def, Vec::new()), args: vec![loc_op, a, b, rem], dest: Place::local(tmp), next, span }, span);
        self.cur = next;
        let wide = Operand::Move(Place::local(tmp));
        if ty == Ty::Int(IntTy::I64) {
            self.assign(dest, Rvalue::Use(wide), span);
        } else {
            self.assign(dest, Rvalue::Cast(wide, ty), span);
        }
    }

    pub fn lower_wide_mul(&mut self, l: &Expr, r: &Expr, dest: &Place, ty: Ty, span: Span) {
        let name = if ty == Ty::Int(IntTy::I128) { "checked_mul128" } else { "checked_umul128" };
        self.lower_wide_op(name, false, BinOp::Mul, l, r, dest, ty, span);
    }

    /// `rem` adds the bool that `div128` and `udiv128` take. The operands are not widened.
    fn lower_wide_op(&mut self, name: &str, rem: bool, op: BinOp, l: &Expr, r: &Expr, dest: &Place, ty: Ty, span: Span) {
        let def = self.sys_fn(name);
        let a = self.as_operand(l);
        let b = self.as_operand(r);
        let (line, col) = self.sm.line_col(span);
        let loc = format!("{}|{}|{}", self.sm.path(span.file), line, col);
        let mut args = vec![Operand::Const(Const::Str(loc.into_bytes()), self.ref_str()), a, b];
        if rem {
            args.push(Operand::Const(Const::Bool(op == BinOp::Rem), Ty::Bool));
        }
        let tmp = self.temp(ty, span);
        let next = self.new_block();
        self.terminate(Term::Call { func: Func::Def(def, Vec::new()), args, dest: Place::local(tmp), next, span }, span);
        self.cur = next;
        self.assign(dest, Rvalue::Use(Operand::Move(Place::local(tmp))), span);
    }

    fn sys_fn(&self, name: &str) -> DefId {
        self.prog
            .modules
            .iter()
            .find(|m| m.path == "std.sys")
            .and_then(|m| m.items.get(name).copied())
            .unwrap_or_else(|| panic!("std.sys {name}"))
    }

    fn checked_div_id(&mut self) -> DefId {
        if let Some(d) = self.checked_div {
            return d;
        }
        let d = self
            .prog
            .modules
            .iter()
            .find(|m| m.path == "std.sys")
            .and_then(|m| m.items.get("checked_div").copied())
            .expect("std.sys checked_div");
        self.checked_div = Some(d);
        d
    }

    fn ref_str(&self) -> Ty {
        let d = self.prog.prelude_type("str").expect("std str");
        Ty::Ref { mutable: false, lt: None, inner: Box::new(Ty::Adt(d, Vec::new())) }
    }

    /// Evaluates `e` and sign-extends or zero-extends it to `i64`.
    fn widen_i64(&mut self, e: &Expr) -> Operand {
        let op = self.as_operand(e);
        if matches!(e.ty, Ty::Int(IntTy::I64)) {
            return op;
        }
        let t = self.temp(Ty::Int(IntTy::I64), e.span);
        self.assign(&Place::local(t), Rvalue::Cast(op, Ty::Int(IntTy::I64)), e.span);
        Operand::Move(Place::local(t))
    }
}
