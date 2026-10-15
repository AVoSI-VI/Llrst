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

    pub fn binop(&mut self, op: BinOp, x: Value, y: Value, t: &Ty) -> Value {
        let b = &mut self.b;
        if matches!(t, Ty::F32 | Ty::F64) {
            return match op {
                BinOp::Add => b.ins().fadd(x, y),
                BinOp::Sub => b.ins().fsub(x, y),
                BinOp::Mul => b.ins().fmul(x, y),
                BinOp::Div => b.ins().fdiv(x, y),
                BinOp::Eq => b.ins().fcmp(FloatCC::Equal, x, y),
                BinOp::Ne => b.ins().fcmp(FloatCC::NotEqual, x, y),
                BinOp::Lt => b.ins().fcmp(FloatCC::LessThan, x, y),
                BinOp::Le => b.ins().fcmp(FloatCC::LessThanOrEqual, x, y),
                BinOp::Gt => b.ins().fcmp(FloatCC::GreaterThan, x, y),
                BinOp::Ge => b.ins().fcmp(FloatCC::GreaterThanOrEqual, x, y),
                _ => panic!("float op {:?}", op),
            };
        }
        let signed = matches!(t, Ty::Int(i) if i.signed());
        match op {
            BinOp::Add => b.ins().iadd(x, y),
            BinOp::Sub => b.ins().isub(x, y),
            BinOp::Mul => b.ins().imul(x, y),
            BinOp::Div => {
                if signed {
                    b.ins().sdiv(x, y)
                } else {
                    b.ins().udiv(x, y)
                }
            }
            BinOp::Rem => {
                if signed {
                    b.ins().srem(x, y)
                } else {
                    b.ins().urem(x, y)
                }
            }
            BinOp::Eq => b.ins().icmp(IntCC::Equal, x, y),
            BinOp::Ne => b.ins().icmp(IntCC::NotEqual, x, y),
            BinOp::Lt => b.ins().icmp(if signed { IntCC::SignedLessThan } else { IntCC::UnsignedLessThan }, x, y),
            BinOp::Le => b.ins().icmp(if signed { IntCC::SignedLessThanOrEqual } else { IntCC::UnsignedLessThanOrEqual }, x, y),
            BinOp::Gt => b.ins().icmp(if signed { IntCC::SignedGreaterThan } else { IntCC::UnsignedGreaterThan }, x, y),
            BinOp::Ge => b.ins().icmp(if signed { IntCC::SignedGreaterThanOrEqual } else { IntCC::UnsignedGreaterThanOrEqual }, x, y),
            BinOp::BitAnd => b.ins().band(x, y),
            BinOp::BitOr => b.ins().bor(x, y),
            BinOp::BitXor => b.ins().bxor(x, y),
            BinOp::Shl => b.ins().ishl(x, y),
            BinOp::Shr => {
                if signed {
                    b.ins().sshr(x, y)
                } else {
                    b.ins().ushr(x, y)
                }
            }
        }
    }

    pub fn cast(&mut self, x: Value, from: &Ty, to: &Ty) -> Value {
        let b = &mut self.b;
        match (from, to) {
            (Ty::Int(f), Ty::Int(t)) => {
                if f.bytes() == t.bytes() {
                    x
                } else if f.bytes() > t.bytes() {
                    b.ins().ireduce(clif_type(to).unwrap(), x)
                } else if f.signed() {
                    b.ins().sextend(clif_type(to).unwrap(), x)
                } else {
                    b.ins().uextend(clif_type(to).unwrap(), x)
                }
            }
            (Ty::Bool, Ty::Int(t)) => {
                if t.bytes() == 1 {
                    x
                } else {
                    b.ins().uextend(clif_type(to).unwrap(), x)
                }
            }
            (Ty::Int(f), Ty::F32 | Ty::F64) => {
                let ft = clif_type(to).unwrap();
                if f.signed() {
                    b.ins().fcvt_from_sint(ft, x)
                } else {
                    b.ins().fcvt_from_uint(ft, x)
                }
            }
            (Ty::F32 | Ty::F64, Ty::Int(t)) => {
                let it = clif_type(to).unwrap();
                if t.signed() {
                    b.ins().fcvt_to_sint_sat(it, x)
                } else {
                    b.ins().fcvt_to_uint_sat(it, x)
                }
            }
            (Ty::F32, Ty::F64) => b.ins().fpromote(types::F64, x),
            (Ty::F64, Ty::F32) => b.ins().fdemote(types::F32, x),
            _ => x,
        }
    }

}
