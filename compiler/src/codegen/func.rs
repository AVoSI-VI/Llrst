//! Translation of one MIR body to Cranelift IR.
use super::layout::clif_type;
use super::Codegen;
use crate::diag::{Diag, PResult, Span};
use crate::mir::*;
use crate::ty::{IntTy, Ty};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{self, types, InstBuilder, MemFlagsData, StackSlot, StackSlotData, StackSlotKind, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Module};

#[derive(Clone, Copy)]
pub enum Val {
    Scalar(Value),
    /// Address of an aggregate value.
    Agg(Value),
    Unit,
}

pub struct FnCx<'a, 'b, 'c> {
    pub cg: &'a mut Codegen<'c>,
    pub b: FunctionBuilder<'b>,
    pub body: &'a Body,
    pub slots: Vec<Option<StackSlot>>,
    pub flags: Vec<Option<StackSlot>>,
    /// Per-field drop flags for a struct local that has no user `drop` method.
    pub field_flags: Vec<Vec<Option<StackSlot>>>,
    pub blocks: Vec<ir::Block>,
    pub sret: Option<Value>,
    pub trace: Option<StackSlot>,
}

pub fn mf() -> MemFlagsData {
    MemFlagsData::new()
}

/// Drop-needed fields of a struct that has no user `drop` method.
fn field_drops(prog: &crate::resolve::Program, ty: &Ty) -> Option<Vec<bool>> {
    if drop_method(prog, ty).is_some() {
        return None;
    }
    let Ty::Adt(d, args) = ty else { return None };
    let crate::resolve::DefKind::Struct(s) = &prog.def(*d).kind else { return None };
    let mask: Vec<bool> = s.fields.iter().map(|f| needs_drop(prog, &f.1.subst(args))).collect();
    if mask.iter().any(|n| *n) { Some(mask) } else { None }
}

pub fn compile(cg: &mut Codegen, body: &Body, id: FuncId) -> PResult<()> {
    super::clos::prepare(cg, body)?;
    let params: Vec<Ty> = (1..=body.arg_count).map(|i| body.locals[i].ty.clone()).collect();
    let ret = body.locals[0].ty.clone();
    let sig = cg.signature(&params, &ret);
    let mut ctx = cg.module.make_context();
    ctx.func.signature = sig;
    let mut fbc = FunctionBuilderContext::new();
    {
        let b = FunctionBuilder::new(&mut ctx.func, &mut fbc);
        let mut cx = FnCx { cg, b, body, slots: Vec::new(), flags: Vec::new(), field_flags: Vec::new(), blocks: Vec::new(), sret: None, trace: None };
        cx.run()?;
        let cfg = cx.cg.frontend;
        let FnCx { b, .. } = cx;
        b.finalize(cfg);
    }
    cg.module
        .define_function(id, &mut ctx)
        .map_err(|e| Diag::error(body.span, "codegen.define").kv("fn", body.name.clone()).kv("got", format!("{:?}", e)))?;
    Ok(())
}

impl<'a, 'b, 'c> FnCx<'a, 'b, 'c> {
    fn run(&mut self) -> PResult<()> {
        let ptr = self.cg.ptr_ty;
        // stack slots
        for l in &self.body.locals {
            let lay = self.cg.layouts.of(&l.ty);
            let slot = if lay.size == 0 {
                None
            } else {
                let shift = (lay.align.max(1)).trailing_zeros() as u8;
                Some(self.b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, lay.size, shift)))
            };
            self.slots.push(slot);
            let parts = field_drops(self.cg.prog, &l.ty);
            if let Some(mask) = parts {
                let mut row = Vec::new();
                for needs in mask {
                    row.push(if needs {
                        Some(self.b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 1, 0)))
                    } else {
                        None
                    });
                }
                self.field_flags.push(row);
                self.flags.push(None);
            } else {
                let flag = if needs_drop(self.cg.prog, &l.ty) {
                    Some(self.b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 1, 0)))
                } else {
                    None
                };
                self.field_flags.push(Vec::new());
                self.flags.push(flag);
            }
        }
        for _ in &self.body.blocks {
            let blk = self.b.create_block();
            self.blocks.push(blk);
        }
        // prologue
        let entry = self.b.create_block();
        self.b.append_block_params_for_function_params(entry);
        self.b.switch_to_block(entry);
        let params: Vec<Value> = self.b.block_params(entry).to_vec();
        let mut pi = 0;
        let ret_ty = self.body.locals[0].ty.clone();
        if self.cg.is_agg(&ret_ty) {
            self.sret = Some(params[0]);
            pi = 1;
        }
        // initialize all drop flags to 0
        for l in 0..self.body.locals.len() {
            if let Some(f) = self.flags[l] {
                let z = self.b.ins().iconst(types::I8, 0);
                self.b.ins().stack_store(self.cg.ptr_ty, z, f, 0);
            }
            for f in self.field_flags[l].clone() {
                if let Some(s) = f {
                    let z = self.b.ins().iconst(types::I8, 0);
                    self.b.ins().stack_store(self.cg.ptr_ty, z, s, 0);
                }
            }
        }
        for i in 1..=self.body.arg_count {
            let t = self.body.locals[i].ty.clone();
            if t.is_unit() {
                continue;
            }
            let v = params[pi];
            pi += 1;
            let addr = self.local_addr(i as u32);
            if self.cg.is_agg(&t) {
                let size = self.cg.layouts.size(&t);
                self.memcpy(addr, v, size);
            } else {
                self.b.ins().store(mf(), v, addr, 0);
            }
            self.set_flag(i as u32, true);
        }
        self.push_trace();
        let _ = ptr;
        self.b.ins().jump(self.blocks[0], &[]);
        for bi in 0..self.body.blocks.len() {
            self.b.switch_to_block(self.blocks[bi]);
            self.block(bi as u32)?;
        }
        self.b.seal_all_blocks();
        Ok(())
    }

    fn push_trace(&mut self) {
        let ptr = self.cg.ptr_ty;
        let wide = (ptr.bytes() as u32) * 2;
        let slot = self.b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, wide, 3));
        self.trace = Some(slot);
        let name = self.body.name.clone();
        let id = self.cg.cstr_data(name.as_bytes());
        let gv = self.cg.module.declare_data_in_func(self.cg.trace, self.b.func);
        let head = self.b.ins().symbol_value(ptr, gv);
        let prev = self.b.ins().load(ptr, mf(), head, 0);
        self.b.ins().stack_store(ptr, prev, slot, 0);
        let ngv = self.cg.module.declare_data_in_func(id, self.b.func);
        let np = self.b.ins().symbol_value(ptr, ngv);
        self.b.ins().stack_store(ptr, np, slot, ptr.bytes() as i32);
        let addr = self.b.ins().stack_addr(ptr, slot, 0);
        self.b.ins().store(mf(), addr, head, 0);
    }

    // ----- addresses, loads, stores -----

    pub fn local_addr(&mut self, l: LocalIdx) -> Value {
        match self.slots[l as usize] {
            Some(s) => self.b.ins().stack_addr(self.cg.ptr_ty, s, 0),
            None => self.b.ins().iconst(self.cg.ptr_ty, 0),
        }
    }

    pub fn place_addr(&mut self, p: &Place) -> (Value, Ty) {
        let mut addr = self.local_addr(p.local);
        let mut ty = self.body.local_ty(p.local).clone();
        let mut variant: Option<u32> = None;
        for pr in &p.proj {
            match pr {
                Proj::Deref => {
                    addr = self.b.ins().load(self.cg.ptr_ty, mf(), addr, 0);
                    ty = match ty {
                        Ty::Ref { inner, .. } | Ty::Ptr(inner) => *inner,
                        other => panic!("deref of {}", other),
                    };
                }
                Proj::Downcast(v) => variant = Some(*v),
                Proj::Field(i) => {
                    let lay = self.cg.layouts.of(&ty);
                    let off = match variant {
                        Some(v) => lay.variants[v as usize][*i as usize],
                        None => lay.fields[*i as usize],
                    };
                    let fty = field_ty(self.cg.prog, &ty, variant, *i);
                    if off != 0 {
                        addr = self.b.ins().iadd_imm_s(addr, off as i64);
                    }
                    ty = fty;
                    variant = None;
                }
            }
        }
        (addr, ty)
    }

    pub fn load(&mut self, ty: &Ty, addr: Value) -> Val {
        if ty.is_unit() {
            return Val::Unit;
        }
        match clif_type(ty) {
            Some(ct) => Val::Scalar(self.b.ins().load(ct, mf(), addr, 0)),
            None => Val::Agg(addr),
        }
    }

    pub fn store(&mut self, ty: &Ty, addr: Value, v: Val) {
        match v {
            Val::Unit => {}
            Val::Scalar(x) => {
                self.b.ins().store(mf(), x, addr, 0);
            }
            Val::Agg(src) => {
                if src != addr {
                    let size = self.cg.layouts.size(ty);
                    self.memcpy(addr, src, size);
                }
            }
        }
    }

    pub fn memcpy(&mut self, dest: Value, src: Value, size: u32) {
        if size == 0 {
            return;
        }
        let cfg = self.cg.frontend;
        self.b.emit_small_memory_copy(cfg, dest, src, size as u64, 1, 1, true, mf());
    }

    pub fn set_flag(&mut self, l: LocalIdx, on: bool) {
        self.store_flag(self.flags[l as usize], on);
        let row = self.field_flags[l as usize].clone();
        for s in row {
            self.store_flag(s, on);
        }
    }

    pub fn set_field_flag(&mut self, l: LocalIdx, field: u32, on: bool) {
        let slot = self.field_flags[l as usize].get(field as usize).copied().flatten();
        self.store_flag(slot, on);
    }

    fn store_flag(&mut self, slot: Option<StackSlot>, on: bool) {
        if let Some(f) = slot {
            let v = self.b.ins().iconst(types::I8, on as i64);
            self.b.ins().stack_store(self.cg.ptr_ty, v, f, 0);
        }
    }

    // ----- operands -----

    pub fn operand(&mut self, op: &Operand) -> (Val, Ty) {
        match op {
            Operand::Copy(p) | Operand::Move(p) => {
                let (addr, ty) = self.place_addr(p);
                if matches!(op, Operand::Move(_)) && !p.has_deref() {
                    if let Some(i) = p.lone_field() {
                        self.set_field_flag(p.local, i, false);
                    } else {
                        self.set_flag(p.local, false);
                    }
                }
                (self.load(&ty, addr), ty)
            }
            Operand::Const(c, ty) => (self.constant(c, ty), ty.clone()),
        }
    }

    pub fn constant(&mut self, c: &Const, ty: &Ty) -> Val {
        match c {
            Const::Int(v, _) => match ty {
                Ty::F32 => Val::Scalar(self.b.ins().f32const(*v as f32)),
                Ty::F64 => Val::Scalar(self.b.ins().f64const(*v as f64)),
                _ => Val::Scalar(self.b.ins().iconst(clif_type(ty).unwrap_or(types::I64), *v as i64)),
            },
            Const::Float(v, is32) => {
                if *is32 {
                    Val::Scalar(self.b.ins().f32const(*v as f32))
                } else {
                    Val::Scalar(self.b.ins().f64const(*v))
                }
            }
            Const::Bool(b) => Val::Scalar(self.b.ins().iconst(types::I8, *b as i64)),
            Const::Unit => Val::Unit,
            Const::Str(s) => {
                let d = self.cg.str_data(s);
                let gv = self.cg.module.declare_data_in_func(d, self.b.func);
                Val::Scalar(self.b.ins().symbol_value(self.cg.ptr_ty, gv))
            }
            Const::Fn(d, targs) => {
                let id = self.cg.func_id(&(*d, targs.clone()));
                let fr = self.cg.module.declare_func_in_func(id, self.b.func);
                Val::Scalar(self.b.ins().func_addr(self.cg.ptr_ty, fr))
            }
        }
    }

    pub fn scalar(&mut self, op: &Operand) -> (Value, Ty) {
        match self.operand(op) {
            (Val::Scalar(v), t) => (v, t),
            (_, t) => panic!("expected scalar operand of type {}", t),
        }
    }

    // ----- statements -----

    fn block(&mut self, bi: BlockId) -> PResult<()> {
        let blk = &self.body.blocks[bi as usize];
        for (s, sp) in &blk.stmts {
            self.stmt(s, *sp)?;
        }
        self.term(&blk.term, blk.term_span)
    }

    fn stmt(&mut self, s: &Stmt, span: Span) -> PResult<()> {
        match s {
            Stmt::Assign(dest, rv) => self.assign(dest, rv, span),
            Stmt::Drop(p) => self.drop_place(p),
            Stmt::Forget(l) => self.set_flag(*l, false),
            Stmt::StorageDead(_) | Stmt::Nop | Stmt::InlineLoans { .. } => {}
        }
        Ok(())
    }

    fn assign(&mut self, dest: &Place, rv: &Rvalue, span: Span) {
        let _ = span;
        match rv {
            Rvalue::Struct(ty, ops) => {
                let vals = self.spilled_operands(ops);
                let (daddr, _) = self.place_addr(dest);
                let lay = self.cg.layouts.of(ty);
                for (i, (v, t)) in vals.into_iter().enumerate() {
                    let a = self.b.ins().iadd_imm_s(daddr, lay.fields[i] as i64);
                    self.store(&t, a, v);
                }
            }
            Rvalue::Variant(ty, idx, ops) => {
                let vals = self.spilled_operands(ops);
                let (daddr, _) = self.place_addr(dest);
                let lay = self.cg.layouts.of(ty);
                let tag = self.b.ins().iconst(types::I32, *idx as i64);
                self.b.ins().store(mf(), tag, daddr, 0);
                for (i, (v, t)) in vals.into_iter().enumerate() {
                    let a = self.b.ins().iadd_imm_s(daddr, lay.variants[*idx as usize][i] as i64);
                    self.store(&t, a, v);
                }
            }
            Rvalue::Closure { idx, caps } => self.clos_assign(dest, *idx, caps),
            _ => {
                let v = self.rvalue(rv, span);
                let (daddr, dty) = self.place_addr(dest);
                self.store(&dty, daddr, v);
            }
        }
        if dest.is_local() {
            self.set_flag(dest.local, true);
        } else if let Some(i) = dest.lone_field() {
            self.set_field_flag(dest.local, i, true);
        }
    }

    /// Evaluates operands before the destination is written; aggregates are copied to
    /// fresh temporaries so an operand that reads `dest` stays intact.
    pub fn spilled_operands(&mut self, ops: &[Operand]) -> Vec<(Val, Ty)> {
        let vals: Vec<(Val, Ty)> = ops.iter().map(|o| self.operand(o)).collect();
        let mut out = Vec::new();
        for (v, t) in vals {
            if let Val::Agg(src) = v {
                let lay = self.cg.layouts.of(&t);
                let shift = (lay.align.max(1)).trailing_zeros() as u8;
                let slot = self.b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, lay.size.max(1), shift));
                let ta = self.b.ins().stack_addr(self.cg.ptr_ty, slot, 0);
                self.memcpy(ta, src, lay.size);
                out.push((Val::Agg(ta), t));
            } else {
                out.push((v, t));
            }
        }
        out
    }

    fn rvalue(&mut self, rv: &Rvalue, span: Span) -> Val {
        match rv {
            Rvalue::Use(op) => self.operand(op).0,
            Rvalue::Ref { place, .. } => Val::Scalar(self.place_addr(place).0),
            Rvalue::Bin(op, a, b) => {
                let (x, t) = self.scalar(a);
                let (y, _) = self.scalar(b);
                Val::Scalar(self.binop(*op, x, y, &t, span))
            }
            Rvalue::Neg(a) | Rvalue::WNeg(a) => {
                let (x, t) = self.scalar(a);
                let wrapping = matches!(rv, Rvalue::WNeg(_));
                Val::Scalar(if matches!(t, Ty::F32 | Ty::F64) {
                    self.b.ins().fneg(x)
                } else if self.cg.checked && !wrapping {
                    self.checked_neg(x, &t, span)
                } else {
                    self.b.ins().ineg(x)
                })
            }
            Rvalue::Not(a) => {
                let (x, _) = self.scalar(a);
                Val::Scalar(self.b.ins().bxor_imm_s(x, 1))
            }
            Rvalue::Cast(a, to) => {
                let (x, from) = self.scalar(a);
                Val::Scalar(self.cast(x, &from, to, span))
            }
            Rvalue::Discr(p) => {
                let (addr, _) = self.place_addr(p);
                Val::Scalar(self.b.ins().load(types::I32, mf(), addr, 0))
            }
            Rvalue::PtrNull => Val::Scalar(self.b.ins().iconst(self.cg.ptr_ty, 0)),
            Rvalue::PtrAdd(p, n, elem) => {
                let (pv, _) = self.scalar(p);
                let (nv, nt) = self.scalar(n);
                let size = self.cg.layouts.size(elem) as i64;
                let n64 = self.to_i64(nv, &nt);
                let off = self.b.ins().imul_imm_s(n64, size);
                Val::Scalar(self.b.ins().iadd(pv, off))
            }
            Rvalue::PtrEq(a, b) => {
                let (x, _) = self.scalar(a);
                let (y, _) = self.scalar(b);
                Val::Scalar(self.b.ins().icmp(IntCC::Equal, x, y))
            }
            Rvalue::Argc => {
                let gv = self.cg.module.declare_data_in_func(self.cg.argc, self.b.func);
                let p = self.b.ins().symbol_value(self.cg.ptr_ty, gv);
                Val::Scalar(self.b.ins().load(types::I64, mf(), p, 0))
            }
            Rvalue::Argv => {
                let gv = self.cg.module.declare_data_in_func(self.cg.argv, self.b.func);
                let p = self.b.ins().symbol_value(self.cg.ptr_ty, gv);
                Val::Scalar(self.b.ins().load(self.cg.ptr_ty, mf(), p, 0))
            }
            Rvalue::Sizeof(t) => {
                let s = self.cg.layouts.size(t);
                Val::Scalar(self.b.ins().iconst(types::I64, s as i64))
            }
            Rvalue::Vtable(tr, ty) => {
                let id = self.cg.vtable(*tr, ty);
                let gv = self.cg.module.declare_data_in_func(id, self.b.func);
                Val::Scalar(self.b.ins().symbol_value(self.cg.ptr_ty, gv))
            }
            Rvalue::Struct(..) | Rvalue::Variant(..) | Rvalue::Closure { .. } => unreachable!(),
        }
    }

}

pub fn field_ty(prog: &crate::resolve::Program, ty: &Ty, variant: Option<u32>, i: u32) -> Ty {
    let Ty::Adt(d, args) = ty else { panic!("field of {}", ty) };
    match (&prog.def(*d).kind, variant) {
        (crate::resolve::DefKind::Struct(s), _) => s.fields[i as usize].1.subst(args),
        (crate::resolve::DefKind::Enum(e), Some(v)) => e.variants[v as usize].1[i as usize].subst(args),
        _ => panic!("bad field projection"),
    }
}

pub fn int_ty_of(t: &Ty) -> IntTy {
    match t {
        Ty::Int(i) => *i,
        _ => IntTy::I64,
    }
}
