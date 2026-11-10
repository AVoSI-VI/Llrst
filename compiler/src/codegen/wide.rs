//! 128-bit constants and the `--checked` traps that are not Cranelift overflow ops.
//! `iconst.i128` panics in the Cranelift 0.136 verifier, so a constant is `iconcat` of two `i64`s.
//! riscv64 does not lower `sadd_overflow`, `ssub_overflow`, or `usub_overflow` on `I128`.
use super::func::FnCx;
use super::layout::clif_int;
use crate::diag::Span;
use crate::ty::IntTy;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, InstBuilder, Value};

impl<'a, 'b, 'c> FnCx<'a, 'b, 'c> {
    pub fn i128_const(&mut self, v: i128) -> Value {
        let lo = self.b.ins().iconst(types::I64, v as i64);
        let hi = self.b.ins().iconst(types::I64, (v >> 64) as i64);
        self.b.ins().iconcat(lo, hi)
    }

    /// `iconst` for an integer Cranelift type. `I128` is two `i64` halves.
    pub fn iconst_int(&mut self, ct: types::Type, v: i64) -> Value {
        if ct == types::I128 {
            self.i128_const(v as i128)
        } else {
            self.b.ins().iconst(ct, v)
        }
    }

    pub fn wide_add(&mut self, x: Value, y: Value, signed: bool, span: Span) -> Value {
        let sum = self.b.ins().iadd(x, y);
        let flag = if signed { self.sign_add(x, y, sum) } else { self.b.ins().icmp(IntCC::UnsignedLessThan, sum, x) };
        self.guard(sum, flag, span)
    }

    pub fn wide_sub(&mut self, x: Value, y: Value, signed: bool, span: Span) -> Value {
        let diff = self.b.ins().isub(x, y);
        let flag = if signed { self.sign_sub(x, y, diff) } else { self.b.ins().icmp(IntCC::UnsignedGreaterThan, diff, x) };
        self.guard(diff, flag, span)
    }

    /// Signed negation traps on `MIN`. Unsigned negation traps on any non-zero value.
    pub fn wide_neg(&mut self, x: Value, signed: bool, span: Span) -> Value {
        if !signed {
            let zero = self.i128_const(0);
            let flag = self.b.ins().icmp(IntCC::NotEqual, x, zero);
            return self.guard(zero, flag, span);
        }
        let n = self.b.ins().ineg(x);
        let both = self.b.ins().band(x, n);
        let flag = self.top_bit(both);
        self.guard(n, flag, span)
    }

    /// A narrowing cast, or a same-width change of signedness. Equal signedness at 128 bits fits.
    pub fn wide_cast_oob(&mut self, x: Value, from: IntTy, to: IntTy) -> Option<Value> {
        if to.wide() {
            if from.signed() == to.signed() {
                return None;
            }
            let zero = self.i128_const(0);
            return Some(self.b.ins().icmp(IntCC::SignedLessThan, x, zero));
        }
        let narrow = self.b.ins().ireduce(clif_int(to), x);
        let back = if to.signed() { self.b.ins().sextend(types::I128, narrow) } else { self.b.ins().uextend(types::I128, narrow) };
        Some(self.b.ins().icmp(IntCC::NotEqual, back, x))
    }

    /// Signs of `x` and `y` match, and the sum's sign does not.
    fn sign_add(&mut self, x: Value, y: Value, sum: Value) -> Value {
        let xy = self.b.ins().bxor(x, y);
        let nxy = self.b.ins().bnot(xy);
        let xs = self.b.ins().bxor(x, sum);
        let bits = self.b.ins().band(nxy, xs);
        self.top_bit(bits)
    }

    /// Signs of `x` and `y` differ, and the difference's sign differs from `x`.
    fn sign_sub(&mut self, x: Value, y: Value, diff: Value) -> Value {
        let xy = self.b.ins().bxor(x, y);
        let xd = self.b.ins().bxor(x, diff);
        let bits = self.b.ins().band(xy, xd);
        self.top_bit(bits)
    }

    fn top_bit(&mut self, bits: Value) -> Value {
        let sh = self.i128_const(127);
        let bit = self.b.ins().ushr(bits, sh);
        self.b.ins().icmp_imm_s(IntCC::NotEqual, bit, 0)
    }
}
