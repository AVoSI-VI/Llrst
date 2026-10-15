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
use cranelift_module::{FuncId, Module};

pub fn compile_drop(cg: &mut Codegen, t: &Ty, id: FuncId) -> PResult<()> {
    let mut ctx = cg.module.make_context();
    ctx.func.signature = cg.module.declarations().get_function_decl(id).signature.clone();
    let mut fbc = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut ctx.func, &mut fbc);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let addr = b.block_params(entry)[0];
        // user drop first
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
            DefKind::Struct(s) => {
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
                b.ins().return_(&[]);
            }
            DefKind::Enum(e) => {
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
                b.ins().return_(&[]);
            }
            _ => unreachable!(),
        }
        b.seal_all_blocks();
        let cfg = cg.frontend;
        b.finalize(cfg);
    }
    cg.module
        .define_function(id, &mut ctx)
        .map_err(|e| Diag::error(Span::default(), "codegen.define").kv("fn", format!("drop {}", t)).kv("got", e.to_string()))
}
