//! `--emit-dylib` entries. Slots live in `runtime/llr_reload.c`.
use super::Codegen;
use crate::diag::{Diag, PResult, Span};
use cranelift_codegen::ir::{self, types, AbiParam, InstBuilder, Signature};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Linkage, Module};

impl Codegen<'_> {
    pub fn emit_boot(&mut self, entry: FuncId, pump: Option<FuncId>, release: Option<FuncId>) -> PResult<()> {
        let set_prev = self.import_void("llr_reload_set_prev", true);
        let code = self.import_i64("llr_reload_code");
        self.boot_fn(entry, set_prev, code)?;
        self.frame_fn(pump)?;
        self.drop_fn(release)?;
        Ok(())
    }

    fn import_void(&mut self, name: &str, arg: bool) -> FuncId {
        let mut sig = Signature::new(self.call_conv);
        if arg {
            sig.params.push(AbiParam::new(types::I64));
        }
        self.module.declare_function(name, Linkage::Import, &sig).unwrap()
    }

    fn import_i64(&mut self, name: &str) -> FuncId {
        let mut sig = Signature::new(self.call_conv);
        sig.returns.push(AbiParam::new(types::I64));
        self.module.declare_function(name, Linkage::Import, &sig).unwrap()
    }

    fn boot_fn(&mut self, entry: FuncId, set_prev: FuncId, code: FuncId) -> PResult<()> {
        let mut sig = Signature::new(self.call_conv);
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(self.ptr_ty));
        sig.returns.push(AbiParam::new(types::I64));
        let id = self.module.declare_function("llr_ui_boot", Linkage::Export, &sig).unwrap();
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        let mut fbc = FunctionBuilderContext::new();
        {
            let mut b = FunctionBuilder::new(&mut ctx.func, &mut fbc);
            let blk = b.create_block();
            b.append_block_params_for_function_params(blk);
            b.switch_to_block(blk);
            let prev = b.block_params(blk)[0];
            let argc = b.block_params(blk)[1];
            let argv = b.block_params(blk)[2];
            let setf = self.module.declare_func_in_func(set_prev, b.func);
            b.ins().call(setf, &[prev]);
            let gc = self.module.declare_data_in_func(self.argc, b.func);
            let gv = self.module.declare_data_in_func(self.argv, b.func);
            let pc = b.ins().symbol_value(self.ptr_ty, gc);
            let pv = b.ins().symbol_value(self.ptr_ty, gv);
            b.ins().store(ir::MemFlagsData::new(), argc, pc, 0);
            b.ins().store(ir::MemFlagsData::new(), argv, pv, 0);
            let entryf = self.module.declare_func_in_func(entry, b.func);
            b.ins().call(entryf, &[]);
            let codef = self.module.declare_func_in_func(code, b.func);
            let call = b.ins().call(codef, &[]);
            let ret = b.inst_results(call)[0];
            b.ins().return_(&[ret]);
            b.seal_all_blocks();
            b.finalize(self.frontend);
        }
        self.define(id, &mut ctx)
    }

    fn frame_fn(&mut self, pump: Option<FuncId>) -> PResult<()> {
        let sig = Signature::new(self.call_conv);
        let id = self.module.declare_function("llr_ui_frame", Linkage::Export, &sig).unwrap();
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        let mut fbc = FunctionBuilderContext::new();
        {
            let mut b = FunctionBuilder::new(&mut ctx.func, &mut fbc);
            let blk = b.create_block();
            b.switch_to_block(blk);
            if let Some(pump) = pump {
                let f = self.module.declare_func_in_func(pump, b.func);
                b.ins().call(f, &[]);
            }
            b.ins().return_(&[]);
            b.seal_all_blocks();
            b.finalize(self.frontend);
        }
        self.define(id, &mut ctx)
    }

    fn drop_fn(&mut self, release: Option<FuncId>) -> PResult<()> {
        let mut sig = Signature::new(self.call_conv);
        sig.params.push(AbiParam::new(types::I64));
        let id = self.module.declare_function("llr_ui_drop", Linkage::Export, &sig).unwrap();
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        let mut fbc = FunctionBuilderContext::new();
        {
            let mut b = FunctionBuilder::new(&mut ctx.func, &mut fbc);
            let blk = b.create_block();
            b.append_block_params_for_function_params(blk);
            b.switch_to_block(blk);
            if let Some(release) = release {
                let arg = b.block_params(blk)[0];
                let f = self.module.declare_func_in_func(release, b.func);
                b.ins().call(f, &[arg]);
            }
            b.ins().return_(&[]);
            b.seal_all_blocks();
            b.finalize(self.frontend);
        }
        self.define(id, &mut ctx)
    }

    fn define(&mut self, id: FuncId, ctx: &mut cranelift_codegen::Context) -> PResult<()> {
        self.module
            .define_function(id, ctx)
            .map_err(|e| Diag::error(Span::default(), "codegen.define").kv("got", e.to_string()))?;
        Ok(())
    }
}
