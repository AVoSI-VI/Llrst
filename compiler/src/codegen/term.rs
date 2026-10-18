//! Drops and block terminators: branches, switches, calls, returns.
use super::func::{mf, FnCx, Val};
use super::layout::clif_type;
use crate::diag::{PResult, Span};
use crate::mir::*;
use crate::ty::Ty;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, InstBuilder, TrapCode, Value};
use cranelift_module::Module;

impl<'a, 'b, 'c> FnCx<'a, 'b, 'c> {
    pub fn drop_place(&mut self, p: &Place) {
        if p.is_local() && self.field_flags[p.local as usize].iter().any(|s| s.is_some()) {
            self.drop_fields(p.local);
            return;
        }
        let ty = place_ty(self.cg.prog, self.body, p);
        if !needs_drop(self.cg.prog, &ty) {
            return;
        }
        let glue = self.cg.drop_glue(&ty);
        let fr = self.cg.module.declare_func_in_func(glue, self.b.func);
        if p.is_local() {
            if let Some(f) = self.flags[p.local as usize] {
                let flag = self.b.ins().stack_load(self.cg.ptr_ty, types::I8, f, 0);
                let do_drop = self.b.create_block();
                let cont = self.b.create_block();
                self.b.ins().brif(flag, do_drop, &[], cont, &[]);
                self.b.switch_to_block(do_drop);
                let addr = self.local_addr(p.local);
                self.b.ins().call(fr, &[addr]);
                self.set_flag(p.local, false);
                self.b.ins().jump(cont, &[]);
                self.b.switch_to_block(cont);
                return;
            }
        }
        let (addr, _) = self.place_addr(p);
        self.b.ins().call(fr, &[addr]);
    }

    fn drop_fields(&mut self, local: LocalIdx) {
        let ty = self.body.local_ty(local).clone();
        let lay = self.cg.layouts.of(&ty);
        let Ty::Adt(d, args) = &ty else { return };
        let crate::resolve::DefKind::Struct(s) = &self.cg.prog.def(*d).kind else { return };
        let parts: Vec<(usize, Ty, u32)> = s
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (i, f.1.subst(args), lay.fields[i]))
            .collect();
        let base = self.local_addr(local);
        for (i, ft, off) in parts {
            let Some(slot) = self.field_flags[local as usize].get(i).copied().flatten() else { continue };
            if !needs_drop(self.cg.prog, &ft) {
                continue;
            }
            let flag = self.b.ins().stack_load(self.cg.ptr_ty, types::I8, slot, 0);
            let do_drop = self.b.create_block();
            let cont = self.b.create_block();
            self.b.ins().brif(flag, do_drop, &[], cont, &[]);
            self.b.switch_to_block(do_drop);
            let glue = self.cg.drop_glue(&ft);
            let fr = self.cg.module.declare_func_in_func(glue, self.b.func);
            let fa = self.b.ins().iadd_imm_s(base, off as i64);
            self.b.ins().call(fr, &[fa]);
            self.set_field_flag(local, i as u32, false);
            self.b.ins().jump(cont, &[]);
            self.b.switch_to_block(cont);
        }
    }

    // ----- terminators -----

    pub fn term(&mut self, t: &Term, span: Span) -> PResult<()> {
        match t {
            Term::Goto(b) => {
                self.b.ins().jump(self.blocks[*b as usize], &[]);
            }
            Term::If(op, a, c) => {
                let (v, _) = self.scalar(op);
                self.b.ins().brif(v, self.blocks[*a as usize], &[], self.blocks[*c as usize], &[]);
            }
            Term::Switch(op, cases, default) => {
                let (v, _) = self.scalar(op);
                for (val, target) in cases {
                    let c = self.b.ins().icmp_imm_s(IntCC::Equal, v, *val as i64);
                    let next = self.b.create_block();
                    self.b.ins().brif(c, self.blocks[*target as usize], &[], next, &[]);
                    self.b.switch_to_block(next);
                }
                self.b.ins().jump(self.blocks[*default as usize], &[]);
            }
            Term::Call { func, args, dest, next, .. } => self.call(func, args, dest, *next, span)?,
            Term::Return => {
                self.pop_trace();
                let ret_ty = self.body.locals[0].ty.clone();
                if ret_ty.is_unit() {
                    self.b.ins().return_(&[]);
                } else if let Some(sret) = self.sret {
                    let addr = self.local_addr(0);
                    let size = self.cg.layouts.size(&ret_ty);
                    self.memcpy(sret, addr, size);
                    self.b.ins().return_(&[]);
                } else {
                    let addr = self.local_addr(0);
                    let v = self.b.ins().load(clif_type(&ret_ty).unwrap(), mf(), addr, 0);
                    self.b.ins().return_(&[v]);
                }
            }
            Term::Unreachable | Term::None => {
                self.b.ins().trap(TrapCode::user(1).unwrap());
            }
        }
        Ok(())
    }

    fn pop_trace(&mut self) {
        let Some(slot) = self.trace else { return };
        let ptr = self.cg.ptr_ty;
        let prev = self.b.ins().stack_load(ptr, ptr, slot, 0);
        let gv = self.cg.module.declare_data_in_func(self.cg.trace, self.b.func);
        let head = self.b.ins().symbol_value(ptr, gv);
        self.b.ins().store(mf(), prev, head, 0);
    }

    pub fn call(&mut self, func: &Func, args: &[Operand], dest: &Place, next: BlockId, span: Span) -> PResult<()> {
        let _ = span;
        let mut vals: Vec<Value> = Vec::new();
        let (daddr, dty) = self.place_addr(dest);
        let callee_ret = match func {
            Func::Def(d, targs) => self.cg.prog.fn_def(*d).ret.subst(targs),
            Func::Ptr(op) => match op.place().map(|p| place_ty(self.cg.prog, self.body, p)) {
                Some(Ty::Fn(_, r)) => *r,
                _ => dty.clone(),
            },
        };
        if self.cg.is_agg(&callee_ret) {
            vals.push(daddr);
        }
        for a in args {
            match self.operand(a) {
                (Val::Scalar(v), _) | (Val::Agg(v), _) => vals.push(v),
                (Val::Unit, _) => {}
            }
        }
        let inst = match func {
            Func::Def(d, targs) => {
                let id = self.cg.func_id(&(*d, targs.clone()));
                let fr = self.cg.module.declare_func_in_func(id, self.b.func);
                self.b.ins().call(fr, &vals)
            }
            Func::Ptr(op) => {
                let (fv, fty) = self.scalar(op);
                let Ty::Fn(ps, r) = fty else { panic!("call of non-fn") };
                let sig = self.cg.signature(&ps, &r);
                let sr = self.b.import_signature(sig);
                self.b.ins().call_indirect(sr, fv, &vals)
            }
        };
        let results = self.b.inst_results(inst).to_vec();
        if let Some(r) = results.first() {
            if !dty.is_unit() && !callee_ret.is_unit() {
                self.b.ins().store(mf(), *r, daddr, 0);
            }
        }
        if dest.is_local() && !dty.is_unit() && !matches!(callee_ret, Ty::Never) {
            self.set_flag(dest.local, true);
        }
        self.b.ins().jump(self.blocks[next as usize], &[]);
        Ok(())
    }
}
