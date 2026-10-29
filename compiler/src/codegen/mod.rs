//! Cranelift backend: module setup, symbol management, data, ABI.
mod clos;
mod drop;
mod func;
mod ovf;
pub mod layout;
mod ops;
mod term;
mod trace;
mod vtbl;

use crate::diag::{Diag, PResult, SourceMap, Span};
use crate::mir::Body;
use crate::resolve::Program;
use crate::ty::{DefId, Ty};
use cranelift_codegen::ir::{self, types, AbiParam, InstBuilder, Signature};
use cranelift_codegen::isa::{self, CallConv, OwnedTargetIsa, TargetFrontendConfig};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use layout::Layouts;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub type Instance = (DefId, Vec<Ty>);

pub struct Codegen<'a> {
    pub prog: &'a Program,
    pub module: ObjectModule,
    pub layouts: Layouts<'a>,
    pub funcs: HashMap<Instance, FuncId>,
    pub externs: HashMap<DefId, FuncId>,
    pub strs: HashMap<Vec<u8>, DataId>,
    pub scalars: HashMap<Vec<u8>, DataId>,
    pub drops: HashMap<Ty, FuncId>,
    pub pending_drops: Vec<Ty>,
    /// `{body.name}` of a closure -> (trampoline, env drop).
    pub clos_ids: HashMap<String, (FuncId, FuncId)>,
    pub vtables: HashMap<(DefId, Ty), DataId>,
    pub argc: DataId,
    pub argv: DataId,
    /// Head of the panic frame list. Each frame is `{prev, name}`.
    pub trace: DataId,
    pub trace_push: FuncId,
    pub trace_pop: FuncId,
    pub name_syms: usize,
    pub call_conv: CallConv,
    pub ptr_ty: types::Type,
    pub frontend: TargetFrontendConfig,
    pub big_endian: bool,
    pub checked: bool,
    /// Apple libc uses different flag bits, `sockaddr_in`, and `rlimit` numbers.
    pub darwin: bool,
    /// `(path, line starts)` copied from the source map, for panic locations.
    pub locs: Vec<(String, Vec<u32>)>,
    /// Instances discovered while compiling bodies, for the mono driver.
    pub discovered: Vec<Instance>,
}

fn import_trace(module: &mut ObjectModule, call_conv: CallConv, ptr: types::Type, name: &str) -> FuncId {
    let mut sig = Signature::new(call_conv);
    sig.params.push(AbiParam::new(ptr));
    module.declare_function(name, Linkage::Import, &sig).unwrap()
}

pub fn make_isa(target: Option<&str>) -> PResult<OwnedTargetIsa> {
    let mut flags = settings::builder();
    flags.set("opt_level", "speed").unwrap();
    flags.set("is_pic", "true").unwrap();
    let flags = settings::Flags::new(flags);
    let builder = match target {
        None => cranelift_native::builder().map_err(|e| Diag::error(Span::default(), "target.unsupported").kv("got", e))?,
        Some(t) => {
            let triple: target_lexicon::Triple = t.parse().map_err(|_| Diag::error(Span::default(), "target.unsupported").kv("got", t))?;
            isa::lookup(triple).map_err(|e| Diag::error(Span::default(), "target.unsupported").kv("got", e.to_string()))?
        }
    };
    builder.finish(flags).map_err(|e| Diag::error(Span::default(), "target.unsupported").kv("got", e.to_string()))
}

/// Darwin libc rejects the Linux numbers in `std`. The runtime object defines the `llr_*` names.
fn host_symbol<'a>(name: &'a str, darwin: bool) -> &'a str {
    if !darwin {
        return name;
    }
    match name {
        "open" => "llr_open",
        "fcntl" => "llr_fcntl",
        "bind" => "llr_bind",
        "connect" => "llr_connect",
        "socket" => "llr_socket",
        "setrlimit" => "llr_setrlimit",
        _ => name,
    }
}

impl<'a> Codegen<'a> {
    pub fn new(prog: &'a Program, isa: OwnedTargetIsa, sm: &SourceMap, checked: bool) -> PResult<Self> {
        let call_conv = isa.default_call_conv();
        let ptr_ty = isa.pointer_type();
        let frontend = isa.frontend_config();
        let darwin = isa.triple().operating_system.is_like_darwin();
        let big_endian = isa.triple().endianness().map_or(false, |e| e == target_lexicon::Endianness::Big);
        let builder = ObjectBuilder::new(isa, "llr", cranelift_module::default_libcall_names())
            .map_err(|e| Diag::error(Span::default(), "codegen.init").kv("got", e.to_string()))?;
        let mut module = ObjectModule::new(builder);
        let argc = module.declare_data("llr_argc", Linkage::Local, true, false).unwrap();
        let argv = module.declare_data("llr_argv", Linkage::Local, true, false).unwrap();
        let trace = module.declare_data("llr_trace", Linkage::Export, true, false).unwrap();
        let trace_push = import_trace(&mut module, call_conv, ptr_ty, "llr_trace_push");
        let trace_pop = import_trace(&mut module, call_conv, ptr_ty, "llr_trace_pop");
        for d in [argc, argv, trace] {
            let mut desc = DataDescription::new();
            desc.define_zeroinit(8);
            desc.set_align(8);
            module.define_data(d, &desc).unwrap();
        }
        Ok(Codegen {
            prog,
            module,
            layouts: Layouts::new(prog),
            funcs: HashMap::new(),
            externs: HashMap::new(),
            strs: HashMap::new(),
            scalars: HashMap::new(),
            drops: HashMap::new(),
            pending_drops: Vec::new(),
            clos_ids: HashMap::new(),
            vtables: HashMap::new(),
            argc,
            argv,
            trace,
            trace_push,
            trace_pop,
            name_syms: 0,
            call_conv,
            ptr_ty,
            frontend,
            big_endian,
            checked,
            darwin,
            locs: sm.export_locs(),
            discovered: Vec::new(),
        })
    }

    pub fn is_agg(&self, t: &Ty) -> bool {
        !t.is_scalar() && !t.is_unit()
    }

    /// Signature for an Llrst function: aggregates by pointer, aggregate returns via sret.
    pub fn signature(&mut self, params: &[Ty], ret: &Ty) -> Signature {
        let mut sig = Signature::new(self.call_conv);
        if self.is_agg(ret) {
            sig.params.push(AbiParam::new(self.ptr_ty));
        }
        for p in params {
            if p.is_unit() {
                continue;
            }
            let t = layout::clif_type(p).unwrap_or(self.ptr_ty);
            sig.params.push(AbiParam::new(t));
        }
        if let Some(t) = layout::clif_type(ret) {
            sig.returns.push(AbiParam::new(t));
        }
        sig
    }

    pub fn mangle(&self, inst: &Instance) -> String {
        let path = self.prog.fn_path(inst.0).replace('.', "_");
        if inst.1.is_empty() {
            return format!("llr_{}", path);
        }
        let mut h = DefaultHasher::new();
        inst.1.hash(&mut h);
        format!("llr_{}_{:x}", path, h.finish())
    }

    /// Declares (once) and returns the FuncId for an instance, recording it for compilation.
    pub fn func_id(&mut self, inst: &Instance) -> FuncId {
        if let Some(id) = self.funcs.get(inst) {
            return *id;
        }
        let fd = self.prog.fn_def(inst.0);
        if fd.is_extern {
            return self.extern_id(inst.0);
        }
        let params: Vec<Ty> = fd.params.iter().map(|p| p.1.subst(&inst.1)).collect();
        let ret = fd.ret.subst(&inst.1);
        let sig = self.signature(&params, &ret);
        let name = self.mangle(inst);
        let id = self.module.declare_function(&name, Linkage::Local, &sig).unwrap();
        self.funcs.insert(inst.clone(), id);
        self.discovered.push(inst.clone());
        id
    }

    pub fn extern_id(&mut self, def: DefId) -> FuncId {
        if let Some(id) = self.externs.get(&def) {
            return *id;
        }
        let fd = self.prog.fn_def(def);
        let params: Vec<Ty> = fd.params.iter().map(|p| p.1.clone()).collect();
        let mut sig = Signature::new(self.call_conv);
        for p in &params {
            sig.params.push(AbiParam::new(layout::clif_type(p).unwrap_or(self.ptr_ty)));
        }
        if let Some(t) = layout::clif_type(&fd.ret) {
            sig.returns.push(AbiParam::new(t));
        }
        let owned = self.prog.def(def).name.clone();
        let name = host_symbol(&owned, self.darwin);
        let id = self.module.declare_function(name, Linkage::Import, &sig).unwrap();
        self.externs.insert(def, id);
        id
    }

    /// Static `str` struct for a literal: `{ptr, len, cap=0}` pointing at the bytes.
    pub fn str_data(&mut self, bytes: &[u8]) -> DataId {
        if let Some(id) = self.strs.get(bytes) {
            return *id;
        }
        let n = self.strs.len();
        let bytes_id = self.module.declare_data(&format!("llr_sb{}", n), Linkage::Local, false, false).unwrap();
        let mut bd = DataDescription::new();
        let mut content = bytes.to_vec();
        content.push(0);
        bd.define(content.into_boxed_slice());
        bd.set_align(1);
        self.module.define_data(bytes_id, &bd).unwrap();
        let str_id = self.module.declare_data(&format!("llr_ss{}", n), Linkage::Local, false, false).unwrap();
        let mut sd = DataDescription::new();
        let mut buf = vec![0u8; 24];
        let len = bytes.len() as u64;
        buf[8..16].copy_from_slice(&if self.big_endian { len.to_be_bytes() } else { len.to_le_bytes() });
        sd.define(buf.into_boxed_slice());
        sd.set_align(8);
        let gv = self.module.declare_data_in_data(bytes_id, &mut sd);
        sd.write_data_addr(0, gv, 0);
        self.module.define_data(str_id, &sd).unwrap();
        self.strs.insert(bytes.to_vec(), str_id);
        str_id
    }

    /// Addressable bytes of a const scalar.
    pub fn scalar_data(&mut self, c: &crate::mir::Const) -> DataId {
        let (bytes, align) = scalar_bytes(c, self.big_endian);
        if let Some(id) = self.scalars.get(&bytes) {
            return *id;
        }
        let n = self.scalars.len();
        let id = self.module.declare_data(&format!("llr_k{}", n), Linkage::Local, false, false).unwrap();
        let mut bd = DataDescription::new();
        bd.define(bytes.clone().into_boxed_slice());
        bd.set_align(align);
        self.module.define_data(id, &bd).unwrap();
        self.scalars.insert(bytes, id);
        id
    }

    /// NUL-terminated bytes for a panic frame name.
    pub fn cstr_data(&mut self, bytes: &[u8]) -> DataId {
        let n = self.name_syms;
        self.name_syms += 1;
        let id = self.module.declare_data(&format!("llr_nm{}", n), Linkage::Local, false, false).unwrap();
        let mut bd = DataDescription::new();
        let mut content = bytes.to_vec();
        content.push(0);
        bd.define(content.into_boxed_slice());
        bd.set_align(1);
        self.module.define_data(id, &bd).unwrap();
        id
    }

    /// Drop glue function for a type, declared on first use and generated later.
    pub fn drop_glue(&mut self, t: &Ty) -> FuncId {
        if let Some(id) = self.drops.get(t) {
            return *id;
        }
        let mut sig = Signature::new(self.call_conv);
        sig.params.push(AbiParam::new(self.ptr_ty));
        let mut h = DefaultHasher::new();
        t.hash(&mut h);
        let name = format!("llr_drop_{:x}", h.finish());
        let id = self.module.declare_function(&name, Linkage::Local, &sig).unwrap();
        self.drops.insert(t.clone(), id);
        self.pending_drops.push(t.clone());
        id
    }

    pub fn compile_body(&mut self, body: &Body, inst: &Instance) -> PResult<()> {
        let id = self.func_id(inst);
        func::compile(self, body, id)
    }

    pub fn compile_body_with_id(&mut self, body: &Body, id: FuncId) -> PResult<()> {
        func::compile(self, body, id)
    }

    pub fn finish_drops(&mut self) -> PResult<()> {
        while let Some(t) = self.pending_drops.pop() {
            let id = self.drops[&t];
            drop::compile_drop(self, &t, id)?;
        }
        Ok(())
    }

    /// Emits the C `main` that records argv and calls the Llrst entry.
    pub fn emit_entry(&mut self, entry: FuncId) -> PResult<()> {
        let mut sig = Signature::new(self.call_conv);
        sig.params.push(AbiParam::new(types::I32));
        sig.params.push(AbiParam::new(self.ptr_ty));
        sig.returns.push(AbiParam::new(types::I32));
        let id = self.module.declare_function("main", Linkage::Export, &sig).unwrap();
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        let mut fbc = cranelift_frontend::FunctionBuilderContext::new();
        {
            let mut b = cranelift_frontend::FunctionBuilder::new(&mut ctx.func, &mut fbc);
            let blk = b.create_block();
            b.append_block_params_for_function_params(blk);
            b.switch_to_block(blk);
            let argc = b.block_params(blk)[0];
            let argv = b.block_params(blk)[1];
            let argc64 = b.ins().sextend(types::I64, argc);
            let gc = self.module.declare_data_in_func(self.argc, b.func);
            let gv = self.module.declare_data_in_func(self.argv, b.func);
            let pc = b.ins().symbol_value(self.ptr_ty, gc);
            let pv = b.ins().symbol_value(self.ptr_ty, gv);
            b.ins().store(ir::MemFlagsData::new(), argc64, pc, 0);
            b.ins().store(ir::MemFlagsData::new(), argv, pv, 0);
            let f = self.module.declare_func_in_func(entry, b.func);
            b.ins().call(f, &[]);
            let zero = b.ins().iconst(types::I32, 0);
            b.ins().return_(&[zero]);
            b.seal_all_blocks();
            b.finalize(self.frontend);
        }
        self.module.define_function(id, &mut ctx).map_err(|e| Diag::error(Span::default(), "codegen.define").kv("got", e.to_string()))?;
        Ok(())
    }

    /// Emits a test entry: calls `test.run(name, fn)` for each test, then `test.done()`.
    pub fn emit_test_entry(&mut self, tests: &[(String, FuncId)], run: FuncId, done: FuncId) -> PResult<FuncId> {
        let sig = Signature::new(self.call_conv);
        let id = self.module.declare_function("llr_test_entry", Linkage::Local, &sig).unwrap();
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        let mut fbc = cranelift_frontend::FunctionBuilderContext::new();
        {
            let mut b = cranelift_frontend::FunctionBuilder::new(&mut ctx.func, &mut fbc);
            let blk = b.create_block();
            b.switch_to_block(blk);
            let runf = self.module.declare_func_in_func(run, b.func);
            let mut fails = b.ins().iconst(types::I64, 0);
            for (name, fid) in tests {
                let sd = self.str_data(name.as_bytes());
                let gv = self.module.declare_data_in_func(sd, b.func);
                let sp = b.ins().symbol_value(self.ptr_ty, gv);
                let fr = self.module.declare_func_in_func(*fid, b.func);
                let fa = b.ins().func_addr(self.ptr_ty, fr);
                let call = b.ins().call(runf, &[sp, fa]);
                let r = b.inst_results(call)[0];
                fails = b.ins().iadd(fails, r);
            }
            let donef = self.module.declare_func_in_func(done, b.func);
            b.ins().call(donef, &[fails]);
            b.ins().return_(&[]);
            b.seal_all_blocks();
            b.finalize(self.frontend);
        }
        self.module.define_function(id, &mut ctx).map_err(|e| Diag::error(Span::default(), "codegen.define").kv("got", e.to_string()))?;
        Ok(id)
    }

    pub fn declare_test_fn(&mut self, i: usize) -> FuncId {
        let sig = Signature::new(self.call_conv);
        self.module.declare_function(&format!("llr_test_{}", i), Linkage::Local, &sig).unwrap()
    }

    pub fn emit_object(self) -> PResult<Vec<u8>> {
        let product = self.module.finish();
        product.emit().map_err(|e| Diag::error(Span::default(), "codegen.emit").kv("got", e.to_string()))
    }
}

fn scalar_bytes(c: &crate::mir::Const, big: bool) -> (Vec<u8>, u64) {
    use crate::mir::Const;
    match c {
        Const::Bool(b) => (vec![*b as u8], 1),
        Const::Int(v, ty) => {
            let n = ty.bytes() as usize;
            let raw = if big { (*v as u128).to_be_bytes() } else { (*v as u128).to_le_bytes() };
            let bytes = if big { raw[16 - n..].to_vec() } else { raw[..n].to_vec() };
            (bytes, n as u64)
        }
        Const::Float(v, is32) => {
            if *is32 {
                let f = *v as f32;
                let b = if big { f.to_be_bytes().to_vec() } else { f.to_le_bytes().to_vec() };
                (b, 4)
            } else {
                let b = if big { v.to_be_bytes().to_vec() } else { v.to_le_bytes().to_vec() };
                (b, 8)
            }
        }
        _ => (vec![0], 1),
    }
}
