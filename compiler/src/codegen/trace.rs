//! Panic frames. Inline frames call `llr_trace_push`. On x86-64 SysV the
//! prologue does not: the walker maps a return address through `llr_fnspans`.
use super::Codegen;
use crate::diag::{Diag, PResult, Span};
use crate::mir::{Body, Stmt};
use cranelift_codegen::ir::{InstBuilder, StackSlot, StackSlotData, StackSlotKind};
use cranelift_codegen::Context;
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{DataDescription, FuncId, Linkage, Module};

/// Linux x86-64. Other targets keep the prologue call so their walker stays valid.
pub fn fast(cg: &Codegen) -> bool {
    if cg.darwin || cg.call_conv != cranelift_codegen::isa::CallConv::SystemV {
        return false;
    }
    matches!(cg.module.isa().triple().architecture, target_lexicon::Architecture::X86_64)
}

/// `None` when the prologue frame is a return-address span instead of a call.
pub fn prologue(cg: &mut Codegen, b: &mut FunctionBuilder, name: &str) -> Option<StackSlot> {
    if fast(cg) {
        return None;
    }
    let slot = alloc_slot(b, cg.ptr_ty.bytes() as u32);
    link(cg, b, slot, name);
    Some(slot)
}

pub fn define(cg: &mut Codegen, id: FuncId, ctx: &mut Context, name: &str, span: Span) -> PResult<()> {
    cg.module
        .define_function(id, ctx)
        .map_err(|e| Diag::error(span, "codegen.define").kv("fn", name).kv("got", format!("{:?}", e)))?;
    if !fast(cg) {
        return Ok(());
    }
    let Some(code) = ctx.compiled_code() else { return Ok(()) };
    let len = code.buffer.data().len() as u32;
    if len == 0 {
        return Ok(());
    }
    let nid = cg.cstr_data(name.as_bytes());
    cg.spans.push((id, nid, len));
    Ok(())
}

pub fn emit_spans(cg: &mut Codegen) {
    if !fast(cg) {
        return;
    }
    let rows: Vec<(FuncId, cranelift_module::DataId, u32)> = cg.spans.clone();
    let n = rows.len();
    let id = cg.module.declare_data("llr_fnspans", Linkage::Export, false, false).unwrap();
    let mut d = DataDescription::new();
    let mut buf = vec![0u8; if n == 0 { 8 } else { n * 24 }];
    for (i, (_, _, len)) in rows.iter().enumerate() {
        let at = i * 24 + 8;
        let bytes = if cg.big_endian { len.to_be_bytes() } else { len.to_le_bytes() };
        buf[at..at + 4].copy_from_slice(&bytes);
    }
    d.define(buf.into_boxed_slice());
    d.set_align(8);
    for (i, (fid, nid, _)) in rows.iter().enumerate() {
        let at = (i * 24) as u32;
        let fr = cg.module.declare_func_in_data(*fid, &mut d);
        d.write_function_addr(at, fr);
        let gv = cg.module.declare_data_in_data(*nid, &mut d);
        d.write_data_addr(at + 16, gv, 0);
    }
    cg.module.define_data(id, &d).unwrap();
    let nid = cg.module.declare_data("llr_fnspans_n", Linkage::Export, false, false).unwrap();
    let mut nd = DataDescription::new();
    let count = n as u32;
    let bytes = if cg.big_endian { count.to_be_bytes() } else { count.to_le_bytes() };
    nd.define(bytes.to_vec().into_boxed_slice());
    nd.set_align(4);
    cg.module.define_data(nid, &nd).unwrap();
}

pub fn alloc_slot(b: &mut FunctionBuilder, ptr_bytes: u32) -> StackSlot {
    b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, ptr_bytes * 2, 3))
}

fn call_slot(cg: &mut Codegen, b: &mut FunctionBuilder, slot: StackSlot, id: cranelift_module::FuncId) {
    let ptr = cg.ptr_ty;
    let addr = b.ins().stack_addr(ptr, slot, 0);
    let fr = cg.module.declare_func_in_func(id, b.func);
    b.ins().call(fr, &[addr]);
}

pub fn link(cg: &mut Codegen, b: &mut FunctionBuilder, slot: StackSlot, name: &str) {
    let ptr = cg.ptr_ty;
    let id = cg.cstr_data(name.as_bytes());
    let ngv = cg.module.declare_data_in_func(id, b.func);
    let np = b.ins().symbol_value(ptr, ngv);
    b.ins().stack_store(ptr, np, slot, ptr.bytes() as i32);
    let push = cg.trace_push;
    call_slot(cg, b, slot, push);
}

pub fn unlink(cg: &mut Codegen, b: &mut FunctionBuilder, slot: StackSlot) {
    let pop = cg.trace_pop;
    call_slot(cg, b, slot, pop);
}

fn inline_count(body: &Body) -> u32 {
    let mut n = 0;
    for blk in &body.blocks {
        for (s, _) in &blk.stmts {
            if let Stmt::TracePush { slot, .. } = s {
                n = n.max(slot + 1);
            }
        }
    }
    n
}

pub fn alloc_inline(cx: &mut super::func::FnCx) {
    let n = inline_count(cx.body);
    let bytes = cx.cg.ptr_ty.bytes() as u32;
    for _ in 0..n {
        let slot = alloc_slot(&mut cx.b, bytes);
        cx.inline_trace.push(slot);
    }
}

pub fn emit_stmt(cx: &mut super::func::FnCx, s: &Stmt) {
    match s {
        Stmt::TracePush { name, slot } => {
            let sl = cx.inline_trace[*slot as usize];
            link(cx.cg, &mut cx.b, sl, name);
        }
        Stmt::TracePop { slot } => {
            let sl = cx.inline_trace[*slot as usize];
            unlink(cx.cg, &mut cx.b, sl);
        }
        _ => {}
    }
}
