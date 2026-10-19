//! Heap environment, trampoline, and env drop for an owned closure.
use super::func::{mf, FnCx};
use super::Codegen;
use crate::diag::PResult;
use crate::mir::{needs_drop, Body, ClosInst, Operand, Place};
use crate::resolve::Program;
use crate::ty::{DefId, IntTy, Ty};
use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Linkage, Module};

fn env_ty() -> Ty {
    Ty::Ptr(Box::new(Ty::Int(IntTy::U8)))
}

fn sys_fn(prog: &Program, name: &str) -> DefId {
    prog.modules
        .iter()
        .find(|m| m.path == "std.sys")
        .and_then(|m| m.items.get(name).copied())
        .unwrap_or_else(|| panic!("std.sys {name}"))
}

pub fn prepare(cg: &mut Codegen, body: &Body) -> PResult<()> {
    for c in &body.closures {
        let params: Vec<Ty> = (1..=c.body.arg_count).map(|i| c.body.locals[i].ty.clone()).collect();
        let ret = c.body.locals[0].ty.clone();
        let sig = cg.signature(&params, &ret);
        let name = format!("llr_{}", c.body.name.replace('.', "_"));
        let id = cg.module.declare_function(&name, Linkage::Local, &sig).unwrap();
        super::func::compile(cg, &c.body, id)?;
        let tramp = emit_tramp(cg, c, id)?;
        let drop = emit_env_drop(cg, c)?;
        cg.clos_ids.insert(c.body.name.clone(), (tramp, drop));
    }
    Ok(())
}

fn emit_tramp(cg: &mut Codegen, c: &ClosInst, inner: FuncId) -> PResult<FuncId> {
    let mut formals = vec![env_ty()];
    formals.extend(c.user.iter().cloned());
    let sig = cg.signature(&formals, &c.ret);
    let name = format!("llr_{}_tramp", c.body.name.replace('.', "_"));
    let id = cg.module.declare_function(&name, Linkage::Local, &sig).unwrap();
    let (_, _, offs) = cg.layouts.seq(&c.caps, 0);
    let mut ctx = cg.module.make_context();
    ctx.func.signature = sig;
    let mut fbc = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut ctx.func, &mut fbc);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let params: Vec<Value> = b.block_params(entry).to_vec();
        let mut pi = 0;
        let mut call_args: Vec<Value> = Vec::new();
        if cg.is_agg(&c.ret) {
            call_args.push(params[0]);
            pi = 1;
        }
        let env = params[pi];
        pi += 1;
        let fr = cg.module.declare_func_in_func(inner, b.func);
        for (ty, off) in c.caps.iter().zip(offs.iter()) {
            if ty.is_unit() {
                continue;
            }
            let addr = b.ins().iadd_imm_s(env, *off as i64);
            if cg.is_agg(ty) {
                call_args.push(addr);
            } else {
                let ct = super::layout::clif_type(ty).unwrap();
                call_args.push(b.ins().load(ct, mf(), addr, 0));
            }
        }
        call_args.extend_from_slice(&params[pi..]);
        let inst = b.ins().call(fr, &call_args);
        let results = b.inst_results(inst).to_vec();
        b.ins().return_(&results);
        b.seal_all_blocks();
        b.finalize(cg.frontend);
    }
    cg.module.define_function(id, &mut ctx).map_err(|e| crate::diag::Diag::error(c.body.span, "codegen.define").kv("fn", name).kv("got", format!("{e:?}")))?;
    Ok(id)
}

fn emit_env_drop(cg: &mut Codegen, c: &ClosInst) -> PResult<FuncId> {
    let mut sig = Signature::new(cg.call_conv);
    sig.params.push(AbiParam::new(cg.ptr_ty));
    let name = format!("llr_{}_envdrop", c.body.name.replace('.', "_"));
    let id = cg.module.declare_function(&name, Linkage::Local, &sig).unwrap();
    let (_, _, offs) = cg.layouts.seq(&c.caps, 0);
    let free = cg.func_id(&(sys_fn(cg.prog, "free"), Vec::new()));
    let mut ctx = cg.module.make_context();
    ctx.func.signature = sig.clone();
    let mut fbc = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut ctx.func, &mut fbc);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let env = b.block_params(entry)[0];
        for (ty, off) in c.caps.iter().zip(offs.iter()) {
            if !needs_drop(cg.prog, ty) {
                continue;
            }
            let g = cg.drop_glue(ty);
            let fr = cg.module.declare_func_in_func(g, b.func);
            let addr = b.ins().iadd_imm_s(env, *off as i64);
            b.ins().call(fr, &[addr]);
        }
        let fr = cg.module.declare_func_in_func(free, b.func);
        b.ins().call(fr, &[env]);
        b.ins().return_(&[]);
        b.seal_all_blocks();
        b.finalize(cg.frontend);
    }
    cg.module.define_function(id, &mut ctx).map_err(|e| crate::diag::Diag::error(c.body.span, "codegen.define").kv("fn", name).kv("got", format!("{e:?}")))?;
    Ok(id)
}

impl<'a, 'b, 'c> FnCx<'a, 'b, 'c> {
    pub fn clos_assign(&mut self, dest: &Place, idx: u32, caps: &[Operand]) {
        let key = format!("{}.clos{}", self.body.name, idx);
        let (tramp, drop) = self.cg.clos_ids[&key];
        let cap_tys = self.body.closures[idx as usize].caps.clone();
        let (size, _, offs) = self.cg.layouts.seq(&cap_tys, 0);
        let alloc = self.cg.func_id(&(sys_fn(self.cg.prog, "alloc"), Vec::new()));
        let n = self.b.ins().iconst(self.cg.ptr_ty, (size as i64).max(1));
        let fr = self.cg.module.declare_func_in_func(alloc, self.b.func);
        let call = self.b.ins().call(fr, &[n]);
        let env = self.b.inst_results(call)[0];
        let vals = self.spilled_operands(caps);
        for ((v, t), off) in vals.into_iter().zip(offs.iter()) {
            let addr = self.b.ins().iadd_imm_s(env, *off as i64);
            self.store(&t, addr, v);
        }
        let (daddr, _) = self.place_addr(dest);
        let tref = self.cg.module.declare_func_in_func(tramp, self.b.func);
        let dref = self.cg.module.declare_func_in_func(drop, self.b.func);
        let tp = self.b.ins().func_addr(self.cg.ptr_ty, tref);
        let dp = self.b.ins().func_addr(self.cg.ptr_ty, dref);
        self.b.ins().store(mf(), tp, daddr, 0);
        self.b.ins().store(mf(), dp, daddr, 8);
        self.b.ins().store(mf(), env, daddr, 16);
    }
}
