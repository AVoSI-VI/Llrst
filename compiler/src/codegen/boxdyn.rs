//! Heap allocation for `(box (dyn Trait))`.
use super::func::{mf, FnCx, Val};
use crate::mir::{Operand, Place};
use crate::resolve::Program;
use crate::ty::{DefId, Ty};
use cranelift_codegen::ir::InstBuilder;
use cranelift_module::Module;

fn sys_fn(prog: &Program, name: &str) -> DefId {
    prog.modules
        .iter()
        .find(|m| m.path == "std.sys")
        .and_then(|m| m.items.get(name).copied())
        .unwrap_or_else(|| panic!("std.sys {name}"))
}

impl<'a, 'b, 'c> FnCx<'a, 'b, 'c> {
    pub fn box_pack(&mut self, dest: &Place, value: &Operand, trait_id: DefId, self_ty: &Ty) {
        let (val, _) = self.operand(value);
        let size = self.cg.layouts.size(self_ty).max(1) as i64;
        let alloc = self.cg.func_id(&(sys_fn(self.cg.prog, "alloc"), Vec::new()));
        let n = self.b.ins().iconst(self.cg.ptr_ty, size);
        let fr = self.cg.module.declare_func_in_func(alloc, self.b.func);
        let call = self.b.ins().call(fr, &[n]);
        let heap = self.b.inst_results(call)[0];
        self.store(self_ty, heap, val);
        let vt = self.cg.vtable(trait_id, self_ty);
        let vd = self.cg.module.declare_data_in_func(vt, self.b.func);
        let vp = self.b.ins().symbol_value(self.cg.ptr_ty, vd);
        let (daddr, _) = self.place_addr(dest);
        self.b.ins().store(mf(), heap, daddr, 0);
        self.b.ins().store(mf(), vp, daddr, 8);
    }

    pub fn box_borrow(&mut self, dest: &Place, place: &Place) {
        let (src, _) = self.place_addr(place);
        let (daddr, dty) = self.place_addr(dest);
        self.store(&dty, daddr, Val::Agg(src));
    }
}
