//! Drop glue: per-type functions that run the user `drop` then drop fields/payload.
use super::func::mf;
use super::Codegen;
use crate::diag::{Diag, PResult, Span};
use crate::mir::{drop_method, needs_drop};
use crate::resolve::DefKind;
use crate::ty::Ty;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, InstBuilder};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_codegen::ir::Block;
use cranelift_module::{FuncId, Module};

pub fn compile_drop(cg: &mut Codegen, t: &Ty, id: FuncId) -> PResult<()> {
    let name = drop_frame(cg, t);
    let mut ctx = cg.module.make_context();
    ctx.func.signature = cg.module.declarations().get_function_decl(id).signature.clone();
    let mut fbc = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut ctx.func, &mut fbc);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let addr = b.block_params(entry)[0];
        let slot = super::trace::alloc_slot(&mut b, cg.ptr_ty.bytes() as u32);
        super::trace::link(cg, &mut b, slot, &name);
        let leave = b.create_block();
        if matches!(t, Ty::Clos(..)) {
            drop_clos(cg, &mut b, addr);
            b.ins().jump(leave, &[]);
        } else {
            drop_adt(cg, &mut b, t, addr, leave);
        }
        b.switch_to_block(leave);
        super::trace::unlink(cg, &mut b, slot);
        b.ins().return_(&[]);
        b.seal_all_blocks();
        let cfg = cg.frontend;
        b.finalize(cfg);
    }
    cg.module
        .define_function(id, &mut ctx)
        .map_err(|e| Diag::error(Span::default(), "codegen.define").kv("fn", format!("drop {}", t)).kv("got", e.to_string()))
}

fn drop_clos(cg: &mut Codegen, b: &mut FunctionBuilder, addr: cranelift_codegen::ir::Value) {
    let ptr = cg.ptr_ty;
    let dropf = b.ins().load(ptr, mf(), addr, 8);
    let env = b.ins().load(ptr, mf(), addr, 16);
    let mut sig = cranelift_codegen::ir::Signature::new(cg.call_conv);
    sig.params.push(cranelift_codegen::ir::AbiParam::new(ptr));
    let sr = b.import_signature(sig);
    b.ins().call_indirect(sr, dropf, &[env]);
}

fn drop_adt(cg: &mut Codegen, b: &mut FunctionBuilder, t: &Ty, addr: cranelift_codegen::ir::Value, leave: Block) {
    if let Some(m) = drop_method(cg.prog, t) {
        let targs = match t {
            Ty::Adt(_, a) => a.clone(),
            _ => Vec::new(),
        };
        let fid = cg.func_id(&(m, targs));
        let fr = cg.module.declare_func_in_func(fid, b.func);
        b.ins().call(fr, &[addr]);
    }
    let Ty::Adt(d, args) = t else { unreachable!() };
    match &cg.prog.def(*d).kind {
        DefKind::Struct(s) => drop_struct(cg, b, t, addr, args, s),
        DefKind::Enum(e) => drop_enum(cg, b, t, addr, args, e),
        _ => unreachable!(),
    }
    b.ins().jump(leave, &[]);
}

fn drop_struct(cg: &mut Codegen, b: &mut FunctionBuilder, t: &Ty, addr: cranelift_codegen::ir::Value, args: &[Ty], s: &crate::resolve::StructDef) {
    let lay = cg.layouts.of(t);
    for (i, f) in s.fields.iter().enumerate() {
        let ft = f.1.subst(args);
        if needs_drop(cg.prog, &ft) {
            let g = cg.drop_glue(&ft);
            let fr = cg.module.declare_func_in_func(g, b.func);
            let fa = b.ins().iadd_imm_s(addr, lay.fields[i] as i64);
            b.ins().call(fr, &[fa]);
        }
    }
}

fn drop_enum(cg: &mut Codegen, b: &mut FunctionBuilder, t: &Ty, addr: cranelift_codegen::ir::Value, args: &[Ty], e: &crate::resolve::EnumDef) {
    let lay = cg.layouts.of(t);
    let tag = b.ins().load(types::I32, mf(), addr, 0);
    let done = b.create_block();
    for (vi, v) in e.variants.iter().enumerate() {
        let ftys: Vec<Ty> = v.1.iter().map(|f| f.subst(args)).collect();
        if !ftys.iter().any(|f| needs_drop(cg.prog, f)) {
            continue;
        }
        let hit = b.create_block();
        let next = b.create_block();
        let c = b.ins().icmp_imm_s(IntCC::Equal, tag, vi as i64);
        b.ins().brif(c, hit, &[], next, &[]);
        b.switch_to_block(hit);
        for (fi, ft) in ftys.iter().enumerate() {
            if needs_drop(cg.prog, ft) {
                let g = cg.drop_glue(ft);
                let fr = cg.module.declare_func_in_func(g, b.func);
                let fa = b.ins().iadd_imm_s(addr, lay.variants[vi][fi] as i64);
                b.ins().call(fr, &[fa]);
            }
        }
        b.ins().jump(done, &[]);
        b.switch_to_block(next);
    }
    b.ins().jump(done, &[]);
    b.switch_to_block(done);
}

fn drop_frame(cg: &Codegen, t: &Ty) -> String {
    match t {
        Ty::Clos(..) => "drop.clos".into(),
        Ty::Adt(d, args) => framed(cg, &cg.prog.type_name(*d), args),
        other => format!("drop.{}", other),
    }
}

fn framed(cg: &Codegen, name: &str, args: &[Ty]) -> String {
    if args.is_empty() {
        format!("drop.{}", name)
    } else {
        let a: Vec<String> = args.iter().map(|x| shown(cg, x)).collect();
        format!("drop.({} {})", name, a.join(" "))
    }
}

fn shown(cg: &Codegen, t: &Ty) -> String {
    match t {
        Ty::Clos(..) => "clos".into(),
        Ty::Adt(d, args) if args.is_empty() => cg.prog.type_name(*d),
        Ty::Adt(d, args) => {
            let a: Vec<String> = args.iter().map(|x| shown(cg, x)).collect();
            format!("({} {})", cg.prog.type_name(*d), a.join(" "))
        }
        other => other.to_string(),
    }
}
