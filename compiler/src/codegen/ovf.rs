//! `--checked` overflow traps: branch to `sys.abort` with `int.overflow` or `cast.overflow`.
use super::func::FnCx;
use crate::ast::BinOp;
use crate::diag::Span;
use crate::ty::{IntTy, Ty};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, InstBuilder, TrapCode, Value};
use cranelift_module::{FuncId, Module};

impl<'a, 'b, 'c> FnCx<'a, 'b, 'c> {
    /// Branches to a panic when `flag` is non-zero. Returns `value` on the continuing path.
    pub fn guard(&mut self, value: Value, flag: Value, span: Span) -> Value {
        let bad = self.b.create_block();
        let ok = self.b.create_block();
        self.b.ins().brif(flag, bad, &[], ok, &[]);
        self.b.switch_to_block(bad);
        self.abort_overflow(span, b"int.overflow");
        self.b.switch_to_block(ok);
        value
    }

    /// Panics when the shift count is outside `0..bitwidth`, then shifts on the continuing path.
    pub fn checked_shift(&mut self, op: BinOp, x: Value, y: Value, t: &Ty, span: Span) -> Value {
        let Ty::Int(it) = t else { return self.emit_shift(op, x, y, false) };
        let ct = int_clif(*it);
        let lim = self.b.ins().iconst(ct, (it.bytes() as i64) * 8);
        let ge = self.b.ins().icmp(if it.signed() { IntCC::SignedGreaterThanOrEqual } else { IntCC::UnsignedGreaterThanOrEqual }, y, lim);
        let flag = if it.signed() {
            let neg = self.b.ins().icmp_imm_s(IntCC::SignedLessThan, y, 0);
            self.b.ins().bor(neg, ge)
        } else {
            ge
        };
        let bad = self.b.create_block();
        let ok = self.b.create_block();
        self.b.ins().brif(flag, bad, &[], ok, &[]);
        self.b.switch_to_block(bad);
        self.abort_overflow(span, b"int.overflow");
        self.b.switch_to_block(ok);
        self.emit_shift(op, x, y, it.signed())
    }

    /// Panics when an integer value does not fit in `to`, then casts on the continuing path.
    pub fn checked_int_cast(&mut self, x: Value, from: IntTy, to: IntTy, span: Span) -> Value {
        let Some(flag) = self.cast_oob(x, from, to) else {
            return self.int_bits(x, from, to);
        };
        let bad = self.b.create_block();
        let ok = self.b.create_block();
        self.b.ins().brif(flag, bad, &[], ok, &[]);
        self.b.switch_to_block(bad);
        self.abort_overflow(span, b"cast.overflow");
        self.b.switch_to_block(ok);
        self.int_bits(x, from, to)
    }

    fn cast_oob(&mut self, x: Value, from: IntTy, to: IntTy) -> Option<Value> {
        if from.signed() == to.signed() && to.bytes() >= from.bytes() {
            return None;
        }
        if !from.signed() && to.signed() && to.bytes() > from.bytes() {
            return None;
        }
        let ct = int_clif(from);
        if from.signed() && !to.signed() {
            let neg = self.b.ins().icmp_imm_s(IntCC::SignedLessThan, x, 0);
            if to.bytes() >= from.bytes() {
                return Some(neg);
            }
            let lim = self.b.ins().iconst(ct, 1i64 << (to.bytes() * 8));
            let hi = self.b.ins().icmp(IntCC::SignedGreaterThanOrEqual, x, lim);
            return Some(self.b.ins().bor(neg, hi));
        }
        if !from.signed() && to.signed() {
            if to.bytes() == from.bytes() {
                return Some(self.b.ins().icmp_imm_s(IntCC::SignedLessThan, x, 0));
            }
            let max = self.b.ins().iconst(ct, (1i64 << ((to.bytes() * 8) - 1)) - 1);
            return Some(self.b.ins().icmp(IntCC::UnsignedGreaterThan, x, max));
        }
        if !to.signed() {
            let lim = self.b.ins().iconst(ct, 1i64 << (to.bytes() * 8));
            return Some(self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, x, lim));
        }
        let bits = (to.bytes() * 8) - 1;
        let min = self.b.ins().iconst(ct, -(1i64 << bits));
        let max = self.b.ins().iconst(ct, (1i64 << bits) - 1);
        let lo = self.b.ins().icmp(IntCC::SignedLessThan, x, min);
        let hi = self.b.ins().icmp(IntCC::SignedGreaterThan, x, max);
        Some(self.b.ins().bor(lo, hi))
    }

    fn emit_shift(&mut self, op: BinOp, x: Value, y: Value, signed: bool) -> Value {
        match op {
            BinOp::Shr if signed => self.b.ins().sshr(x, y),
            BinOp::Shr => self.b.ins().ushr(x, y),
            _ => self.b.ins().ishl(x, y),
        }
    }

    /// Negation that panics on signed MIN and on any non-zero unsigned value.
    pub fn checked_neg(&mut self, x: Value, t: &Ty, span: Span) -> Value {
        let Ty::Int(it) = t else { return self.b.ins().ineg(x) };
        let flag = if it.signed() {
            let min = self.b.ins().iconst(int_clif(*it), signed_min(*it));
            self.b.ins().icmp(IntCC::Equal, x, min)
        } else {
            self.b.ins().icmp_imm_s(IntCC::NotEqual, x, 0)
        };
        let zero = self.b.ins().iconst(int_clif(*it), 0);
        let neg = self.b.ins().ineg(x);
        let result = if it.signed() { neg } else { zero };
        let _ = if it.signed() { zero } else { neg };
        self.guard(result, flag, span)
    }

    fn abort_overflow(&mut self, span: Span, msg: &[u8]) {
        let (path, starts) = &self.cg.locs[span.file as usize];
        let line = match starts.binary_search(&span.lo) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let loc = format!("{}|{}|{}", path, line + 1, span.lo - starts[line] + 1);
        let loc_p = self.str_ptr(loc.as_bytes());
        let msg_p = self.str_ptr(msg);
        let id = self.abort_func();
        let fr = self.cg.module.declare_func_in_func(id, self.b.func);
        self.b.ins().call(fr, &[loc_p, msg_p]);
        self.b.ins().trap(TrapCode::user(1).unwrap());
    }

    fn str_ptr(&mut self, bytes: &[u8]) -> Value {
        let d = self.cg.str_data(bytes);
        let gv = self.cg.module.declare_data_in_func(d, self.b.func);
        self.b.ins().symbol_value(self.cg.ptr_ty, gv)
    }

    fn abort_func(&mut self) -> FuncId {
        let d = self
            .cg
            .prog
            .modules
            .iter()
            .find(|m| m.path == "std.sys")
            .and_then(|m| m.items.get("abort").copied())
            .expect("std.sys abort");
        self.cg.func_id(&(d, Vec::new()))
    }
}

fn int_clif(t: IntTy) -> types::Type {
    match t {
        IntTy::I8 | IntTy::U8 => types::I8,
        IntTy::I16 | IntTy::U16 => types::I16,
        IntTy::I32 | IntTy::U32 => types::I32,
        IntTy::I64 | IntTy::U64 => types::I64,
    }
}

fn signed_min(t: IntTy) -> i64 {
    match t {
        IntTy::I8 => i8::MIN as i64,
        IntTy::I16 => i16::MIN as i64,
        IntTy::I32 => i32::MIN as i64,
        IntTy::I64 => i64::MIN,
        _ => 0,
    }
}
