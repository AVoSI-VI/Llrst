//! Vtables for `(& (dyn Trait))`.
use super::func::{mf, FnCx, Val};
use super::Codegen;
use crate::diag::PResult;
use crate::mir::{Operand, Place};
use crate::traitres::{abi_sig, impl_of};
use crate::ty::{DefId, Ty};
use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_module::{DataDescription, DataId, Linkage, Module};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

impl<'a> Codegen<'a> {
    pub fn vtable(&mut self, tr: DefId, ty: &Ty) -> DataId {
        if let Some(id) = self.vtables.get(&(tr, ty.clone())) {
            return *id;
        }
        let methods = impl_of(self.prog, tr, ty).expect("vtable").clone();
        let fns: Vec<_> = methods.iter().map(|d| self.func_id(&(*d, Vec::new()))).collect();
        let mut h = DefaultHasher::new();
        tr.hash(&mut h);
        ty.hash(&mut h);
        let id = self.module.declare_data(&format!("llr_vt{:x}", h.finish()), Linkage::Local, false, false).unwrap();
        let mut desc = DataDescription::new();
        desc.define(vec![0u8; fns.len() * 8].into_boxed_slice());
        desc.set_align(8);
        for (i, fid) in fns.iter().enumerate() {
            let r = self.module.declare_func_in_data(*fid, &mut desc);
            desc.write_function_addr((i * 8) as u32, r);
        }
        self.module.define_data(id, &desc).unwrap();
        self.vtables.insert((tr, ty.clone()), id);
        id
    }
}

impl<'a, 'b, 'c> FnCx<'a, 'b, 'c> {
    pub fn dyn_call(&mut self, tr: DefId, slot: u32, args: &[Operand], dest: &Place, next: crate::mir::BlockId) -> PResult<()> {
        let (daddr, dty) = self.place_addr(dest);
        let td = self.cg.prog.trait_def(tr).unwrap();
        let (formals, ret) = abi_sig(td, slot as usize);
        let mut vals: Vec<Value> = Vec::new();
        if self.cg.is_agg(&ret) {
            vals.push(daddr);
        }
        let (recv, _) = self.operand(&args[0]);
        let Val::Agg(addr) = recv else { panic!("dyn receiver") };
        let data = self.b.ins().load(self.cg.ptr_ty, mf(), addr, 0);
        let vt = self.b.ins().load(self.cg.ptr_ty, mf(), addr, 8);
        let fp = self.b.ins().load(self.cg.ptr_ty, mf(), vt, (slot * 8) as i32);
        vals.push(data);
        for a in &args[1..] {
            let (v, ty) = self.operand(a);
            let bit = if matches!(ty, Ty::Dyn { .. }) {
                let Val::Agg(p) = v else { panic!("dyn arg") };
                self.b.ins().load(self.cg.ptr_ty, mf(), p, 0)
            } else {
                match v {
                    Val::Scalar(x) | Val::Agg(x) => x,
                    Val::Unit => continue,
                }
            };
            vals.push(bit);
        }
        let sig = self.cg.signature(&formals, &ret);
        let sr = self.b.import_signature(sig);
        let inst = self.b.ins().call_indirect(sr, fp, &vals);
        let results = self.b.inst_results(inst).to_vec();
        if let Some(r) = results.first() {
            if !dty.is_unit() && !ret.is_unit() {
                self.b.ins().store(mf(), *r, daddr, 0);
            }
        }
        if dest.is_local() && !dty.is_unit() && !matches!(ret, Ty::Never) {
            self.set_flag(dest.local, true);
        }
        self.b.ins().jump(self.blocks[next as usize], &[]);
        Ok(())
    }
}
