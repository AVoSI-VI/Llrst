//! Panic frames. Each thread has its own list, updated by `llr_trace_push`.
use super::Codegen;
use crate::mir::{Body, Stmt};
use cranelift_codegen::ir::{InstBuilder, StackSlot, StackSlotData, StackSlotKind};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::Module;

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
