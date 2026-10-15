//! Integer `/` and `%` become a call to `sys.checked_div`, which panics on a zero divisor.
//! The function's own division is lowered raw (`Lower::raw_div`) so the rewrite does not recurse.
use super::lower::Lower;
use super::*;
use crate::ast::BinOp;
use crate::thir::Expr;
use crate::ty::DefId;

impl<'a> Lower<'a> {
    pub fn lower_checked_div(&mut self, op: BinOp, l: &Expr, r: &Expr, dest: &Place, ty: Ty, span: Span) {
        let def = self.checked_div_id();
        let a = self.widen_i64(l);
        let b = self.widen_i64(r);
        let (line, col) = self.sm.line_col(span);
        let loc = format!("{}|{}|{}", self.sm.path(span.file), line, col);
        let loc_op = Operand::Const(Const::Str(loc.into_bytes()), self.ref_str());
        let rem = Operand::Const(Const::Bool(op == BinOp::Rem), Ty::Bool);
        let trap = Operand::Const(Const::Bool(self.checked), Ty::Bool);
        let tmp = self.temp(Ty::Int(IntTy::I64), span);
        let next = self.new_block();
        self.terminate(Term::Call { func: Func::Def(def, Vec::new()), args: vec![loc_op, a, b, rem, trap], dest: Place::local(tmp), next, span }, span);
        self.cur = next;
        let wide = Operand::Move(Place::local(tmp));
        if ty == Ty::Int(IntTy::I64) {
            self.assign(dest, Rvalue::Use(wide), span);
        } else {
            self.assign(dest, Rvalue::Cast(wide, ty), span);
        }
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
