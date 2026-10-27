//! Scalar arithmetic, comparisons and casts.
use super::func::FnCx;
use super::layout::clif_type;
use crate::ast::BinOp;
use crate::ty::Ty;
use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{types, InstBuilder, Value};

impl<'a, 'b, 'c> FnCx<'a, 'b, 'c> {
    pub fn to_i64(&mut self, v: Value, t: &Ty) -> Value {
        match t {
            Ty::Int(i) if i.bytes() < 8 => {
                if i.signed() {
                    self.b.ins().sextend(types::I64, v)
                } else {
                    self.b.ins().uextend(types::I64, v)
                }
            }
            _ => v,
        }
    }

    pub fn binop(&mut self, op: BinOp, x: Value, y: Value, t: &Ty, span: crate::diag::Span) -> Value {
        if matches!(t, Ty::F32 | Ty::F64) {
            return match op {
                BinOp::Add => self.b.ins().fadd(x, y),
                BinOp::Sub => self.b.ins().fsub(x, y),
                BinOp::Mul => self.b.ins().fmul(x, y),
                BinOp::Div => self.b.ins().fdiv(x, y),
                BinOp::Eq => self.b.ins().fcmp(FloatCC::Equal, x, y),
                BinOp::Ne => self.b.ins().fcmp(FloatCC::NotEqual, x, y),
                BinOp::Lt => self.b.ins().fcmp(FloatCC::LessThan, x, y),
                BinOp::Le => self.b.ins().fcmp(FloatCC::LessThanOrEqual, x, y),
                BinOp::Gt => self.b.ins().fcmp(FloatCC::GreaterThan, x, y),
                BinOp::Ge => self.b.ins().fcmp(FloatCC::GreaterThanOrEqual, x, y),
                _ => panic!("float op {:?}", op),
            };
        }
        let signed = matches!(t, Ty::Int(i) if i.signed());
        match op {
            BinOp::Add | BinOp::WAdd => {
                if self.cg.checked && op == BinOp::Add {
                    let (v, of) = if signed { self.b.ins().sadd_overflow(x, y) } else { self.b.ins().uadd_overflow(x, y) };
                    self.guard(v, of, span)
                } else {
                    self.b.ins().iadd(x, y)
                }
            }
            BinOp::Sub | BinOp::WSub => {
                if self.cg.checked && op == BinOp::Sub {
                    let (v, of) = if signed { self.b.ins().ssub_overflow(x, y) } else { self.b.ins().usub_overflow(x, y) };
                    self.guard(v, of, span)
                } else {
                    self.b.ins().isub(x, y)
                }
            }
            BinOp::Mul | BinOp::WMul => {
                if self.cg.checked && op == BinOp::Mul {
                    let (v, of) = if signed { self.b.ins().smul_overflow(x, y) } else { self.b.ins().umul_overflow(x, y) };
                    self.guard(v, of, span)
                } else {
                    self.b.ins().imul(x, y)
                }
            }
            BinOp::Div => {
                if signed {
                    self.b.ins().sdiv(x, y)
                } else {
                    self.b.ins().udiv(x, y)
                }
            }
            BinOp::Rem => {
                if signed {
                    self.b.ins().srem(x, y)
                } else {
                    self.b.ins().urem(x, y)
                }
            }
            BinOp::Eq => self.b.ins().icmp(IntCC::Equal, x, y),
            BinOp::Ne => self.b.ins().icmp(IntCC::NotEqual, x, y),
            BinOp::Lt => self.b.ins().icmp(if signed { IntCC::SignedLessThan } else { IntCC::UnsignedLessThan }, x, y),
            BinOp::Le => self.b.ins().icmp(if signed { IntCC::SignedLessThanOrEqual } else { IntCC::UnsignedLessThanOrEqual }, x, y),
            BinOp::Gt => self.b.ins().icmp(if signed { IntCC::SignedGreaterThan } else { IntCC::UnsignedGreaterThan }, x, y),
            BinOp::Ge => self.b.ins().icmp(if signed { IntCC::SignedGreaterThanOrEqual } else { IntCC::UnsignedGreaterThanOrEqual }, x, y),
            BinOp::BitAnd => self.b.ins().band(x, y),
            BinOp::BitOr => self.b.ins().bor(x, y),
            BinOp::BitXor => self.b.ins().bxor(x, y),
            BinOp::Shl | BinOp::Shr => {
                if self.cg.checked {
                    self.checked_shift(op, x, y, t, span)
                } else {
                    let y = self.mask_shift(y, t);
                    if op == BinOp::Shr && signed {
                        self.b.ins().sshr(x, y)
                    } else if op == BinOp::Shr {
                        self.b.ins().ushr(x, y)
                    } else {
                        self.b.ins().ishl(x, y)
                    }
                }
            }
        }
    }

    /// `count & (bitwidth - 1)`, so an unchecked shift is defined on every target.
    fn mask_shift(&mut self, y: Value, t: &Ty) -> Value {
        let Ty::Int(it) = t else { return y };
        let ct = clif_type(t).expect("int");
        let mask = self.b.ins().iconst(ct, (it.bytes() as i64) * 8 - 1);
        self.b.ins().band(y, mask)
    }

    pub fn cast(&mut self, x: Value, from: &Ty, to: &Ty, span: crate::diag::Span) -> Value {
        match (from, to) {
            (Ty::Int(f), Ty::Int(t)) => {
                if self.cg.checked {
                    self.checked_int_cast(x, *f, *t, span)
                } else {
                    self.int_bits(x, *f, *t)
                }
            }
            (Ty::Bool, Ty::Int(t)) => {
                if t.bytes() == 1 {
                    x
                } else {
                    self.b.ins().uextend(clif_type(to).unwrap(), x)
                }
            }
            (Ty::Int(f), Ty::F32 | Ty::F64) => {
                if self.cg.checked {
                    self.checked_int_to_float(x, *f, to, span)
                } else {
                    self.int_to_float(x, *f, to)
                }
            }
            (Ty::F32 | Ty::F64, Ty::Int(t)) => {
                if self.cg.checked {
                    self.checked_float_to_int(x, from, *t, span)
                } else {
                    self.float_to_int(x, *t, to)
                }
            }
            (Ty::F32, Ty::F64) => self.b.ins().fpromote(types::F64, x),
            (Ty::F64, Ty::F32) => self.b.ins().fdemote(types::F32, x),
            _ => x,
        }
    }

    pub fn int_bits(&mut self, x: Value, f: crate::ty::IntTy, t: crate::ty::IntTy) -> Value {
        let to = Ty::Int(t);
        if f.bytes() == t.bytes() {
            x
        } else if f.bytes() > t.bytes() {
            self.b.ins().ireduce(clif_type(&to).unwrap(), x)
        } else if f.signed() {
            self.b.ins().sextend(clif_type(&to).unwrap(), x)
        } else {
            self.b.ins().uextend(clif_type(&to).unwrap(), x)
        }
    }

    pub fn int_to_float(&mut self, x: Value, from: crate::ty::IntTy, to: &Ty) -> Value {
        let ft = clif_type(to).unwrap();
        if from.signed() {
            self.b.ins().fcvt_from_sint(ft, x)
        } else {
            self.b.ins().fcvt_from_uint(ft, x)
        }
    }

    pub fn float_to_int(&mut self, x: Value, to: crate::ty::IntTy, dst: &Ty) -> Value {
        let it = clif_type(dst).unwrap();
        if to.signed() {
            self.b.ins().fcvt_to_sint_sat(it, x)
        } else {
            self.b.ins().fcvt_to_uint_sat(it, x)
        }
    }

}
