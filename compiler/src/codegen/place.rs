//! Addresses, loads, and stores. An array element is `index * size(T)` bytes in.
use super::func::{mf, FnCx, Val};
use super::layout::clif_type;
use crate::mir::*;
use crate::ty::Ty;
use cranelift_codegen::ir::{types, InstBuilder, StackSlot, Value};

impl<'a, 'b, 'c> FnCx<'a, 'b, 'c> {
    pub fn local_addr(&mut self, l: LocalIdx) -> Value {
        match self.slots[l as usize] {
            Some(s) => self.b.ins().stack_addr(self.cg.ptr_ty, s, 0),
            None => self.b.ins().iconst(self.cg.ptr_ty, 0),
        }
    }

    pub fn place_addr(&mut self, p: &Place) -> (Value, Ty) {
        let mut addr = self.local_addr(p.local);
        let mut ty = self.body.local_ty(p.local).clone();
        let mut variant: Option<u32> = None;
        for pr in &p.proj {
            match pr {
                Proj::Deref => {
                    addr = self.b.ins().load(self.cg.ptr_ty, mf(), addr, 0);
                    ty = match ty {
                        Ty::Ref { inner, .. } | Ty::Ptr(inner) => *inner,
                        other => panic!("deref of {}", other),
                    };
                }
                Proj::Downcast(v) => variant = Some(*v),
                Proj::Index(i) => {
                    let (next, elem) = self.index_addr(addr, *i, &ty);
                    addr = next;
                    ty = elem;
                    variant = None;
                }
                Proj::Field(i) => {
                    let lay = self.cg.layouts.of(&ty);
                    let off = match variant {
                        Some(v) => lay.variants[v as usize][*i as usize],
                        None => lay.fields[*i as usize],
                    };
                    let fty = field_ty(self.cg.prog, &ty, variant, *i);
                    if off != 0 {
                        addr = self.b.ins().iadd_imm_s(addr, off as i64);
                    }
                    ty = fty;
                    variant = None;
                }
            }
        }
        (addr, ty)
    }

    /// `addr + index * size(T)`. The index local is an `i64` loaded from its slot.
    fn index_addr(&mut self, addr: Value, index: LocalIdx, ty: &Ty) -> (Value, Ty) {
        let slot = self.local_addr(index);
        let iv = self.b.ins().load(types::I64, mf(), slot, 0);
        let elem = match ty {
            Ty::Arr(e, _) => e.as_ref().clone(),
            other => panic!("index of {}", other),
        };
        let off = self.b.ins().imul_imm_s(iv, self.cg.layouts.size(&elem) as i64);
        (self.b.ins().iadd(addr, off), elem)
    }

    pub fn load(&mut self, ty: &Ty, addr: Value) -> Val {
        if ty.is_unit() {
            return Val::Unit;
        }
        match clif_type(ty) {
            Some(ct) => Val::Scalar(self.b.ins().load(ct, mf(), addr, 0)),
            None => Val::Agg(addr),
        }
    }

    pub fn store(&mut self, ty: &Ty, addr: Value, v: Val) {
        match v {
            Val::Unit => {}
            Val::Scalar(x) => {
                self.b.ins().store(mf(), x, addr, 0);
            }
            Val::Agg(src) => {
                if src != addr {
                    let size = self.cg.layouts.size(ty);
                    self.memcpy(addr, src, size);
                }
            }
        }
    }

    pub fn memcpy(&mut self, dest: Value, src: Value, size: u32) {
        if size == 0 {
            return;
        }
        let cfg = self.cg.frontend;
        self.b.emit_small_memory_copy(cfg, dest, src, size as u64, 1, 1, true, mf());
    }

    pub fn set_flag(&mut self, l: LocalIdx, on: bool) {
        self.store_flag(self.flags[l as usize], on);
        let row = self.field_flags[l as usize].clone();
        for s in row {
            self.store_flag(s, on);
        }
    }

    pub fn set_field_flag(&mut self, l: LocalIdx, field: u32, on: bool) {
        let slot = self.field_flags[l as usize].get(field as usize).copied().flatten();
        self.store_flag(slot, on);
    }

    fn store_flag(&mut self, slot: Option<StackSlot>, on: bool) {
        if let Some(f) = slot {
            let v = self.b.ins().iconst(types::I8, on as i64);
            self.b.ins().stack_store(self.cg.ptr_ty, v, f, 0);
        }
    }
}

fn field_ty(prog: &crate::resolve::Program, ty: &Ty, variant: Option<u32>, i: u32) -> Ty {
    let Ty::Adt(d, args) = ty else { panic!("field of {}", ty) };
    match (&prog.def(*d).kind, variant) {
        (crate::resolve::DefKind::Struct(s), _) => s.fields[i as usize].1.subst(args),
        (crate::resolve::DefKind::Enum(e), Some(v)) => e.variants[v as usize].1[i as usize].subst(args),
        _ => panic!("bad field projection"),
    }
}
