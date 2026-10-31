//! THIR to MIR lowering.
use super::*;
use crate::diag::{Diag, Diags, SourceMap};
use crate::ast::{BinOp, Lit};
use crate::thir::{Expr, ExprKind, FnBody, LocalId};

pub struct LoopCtx {
    pub brk: BlockId,
    pub cont: BlockId,
    pub scope_depth: usize,
}

#[derive(Default)]
pub struct Scope {
    pub drops: Vec<LocalIdx>,
    pub all: Vec<LocalIdx>,
}

pub struct Lower<'a> {
    pub prog: &'a Program,
    pub diags: &'a Diags,
    pub sm: &'a SourceMap,
    /// Set while lowering `sys.checked_div`, whose own `/` and `%` must stay raw.
    pub raw_div: bool,
    pub checked: bool,
    pub checked_div: Option<DefId>,
    pub body: Body,
    pub cur: BlockId,
    pub local_map: Vec<LocalIdx>,
    pub scopes: Vec<Scope>,
    pub loops: Vec<LoopCtx>,
    pub ret_ty: Ty,
    /// Callees currently being substituted. A recursive call stays a real call.
    pub inline_stack: Vec<crate::ty::DefId>,
    /// Exit block and scope depth of the body being substituted, if any.
    pub inline_exit: Option<(BlockId, usize)>,
    /// Where an inlined `(ret ...)` writes, instead of the caller's return slot.
    pub inline_dest: Option<Place>,
}

pub fn lower_fn(prog: &Program, diags: &Diags, sm: &SourceMap, f: &FnBody, name: String, lifetimes: Vec<String>, checked: bool) -> Body {
    let mut locals = vec![LocalDecl { ty: f.ret.clone(), name: "_0".into(), mutable: true, span: f.span }];
    let mut local_map = Vec::new();
    for l in &f.locals {
        local_map.push(locals.len() as LocalIdx);
        locals.push(LocalDecl { ty: l.ty.clone(), name: l.name.clone(), mutable: l.mutable, span: l.span });
    }
    // Arguments must be the first locals after the return place: reorder by remapping.
    let mut order: Vec<LocalIdx> = f.params.iter().map(|p| local_map[p.0 as usize]).collect();
    for i in 1..locals.len() as LocalIdx {
        if !order.contains(&i) {
            order.push(i);
        }
    }
    let mut remap = vec![0 as LocalIdx; locals.len()];
    let mut new_locals = vec![locals[0].clone()];
    for (new_idx, old) in order.iter().enumerate() {
        remap[*old as usize] = new_idx as LocalIdx + 1;
        new_locals.push(locals[*old as usize].clone());
    }
    for m in local_map.iter_mut() {
        *m = remap[*m as usize];
    }
    let raw_div = name == "sys.checked_div";
    let body = Body { name, locals: new_locals, arg_count: f.params.len(), blocks: Vec::new(), span: f.span, lifetimes, closures: Vec::new() };
    let mut lw = Lower { prog, diags, sm, raw_div, checked, checked_div: None, body, cur: 0, local_map, scopes: Vec::new(), loops: Vec::new(), ret_ty: f.ret.clone(), inline_stack: Vec::new(), inline_exit: None, inline_dest: None };
    let entry = lw.new_block();
    lw.cur = entry;
    lw.push_scope();
    // args that need drop belong to the outermost scope
    for p in &f.params {
        let l = lw.local_map[p.0 as usize];
        if needs_drop(prog, lw.body.local_ty(l)) {
            lw.scopes.last_mut().unwrap().drops.push(l);
        }
    }
    let dest = Place::local(0);
    lw.expr_into(&f.body, dest, f.span);
    lw.pop_scope(f.span);
    lw.terminate(Term::Return, f.span);
    lw.body
}

impl<'a> Lower<'a> {
    pub fn new_block(&mut self) -> BlockId {
        self.body.blocks.push(BasicBlock { stmts: Vec::new(), term: Term::None, term_span: Span::default() });
        self.body.blocks.len() as BlockId - 1
    }
    pub fn push(&mut self, s: Stmt, span: Span) {
        if matches!(self.body.blocks[self.cur as usize].term, Term::None) {
            self.body.blocks[self.cur as usize].stmts.push((s, span));
        }
    }
    pub fn terminate(&mut self, t: Term, span: Span) {
        let b = &mut self.body.blocks[self.cur as usize];
        if matches!(b.term, Term::None) {
            b.term = t;
            b.term_span = span;
        }
    }
    /// Terminates the current block and continues in a fresh one.
    pub fn goto_new(&mut self, span: Span) -> BlockId {
        let b = self.new_block();
        self.terminate(Term::Goto(b), span);
        self.cur = b;
        b
    }
    pub fn temp(&mut self, ty: Ty, span: Span) -> LocalIdx {
        let idx = self.body.locals.len() as LocalIdx;
        self.body.locals.push(LocalDecl { ty: ty.clone(), name: format!("_t{}", idx), mutable: true, span });
        let sc = self.scopes.last_mut().unwrap();
        sc.all.push(idx);
        if needs_drop(self.prog, &ty) {
            sc.drops.push(idx);
        }
        idx
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }
    pub fn pop_scope(&mut self, span: Span) {
        let s = self.scopes.pop().unwrap();
        for l in s.drops.iter().rev() {
            self.push(Stmt::Drop(Place::local(*l)), span);
        }
        for l in s.all.iter() {
            self.push(Stmt::StorageDead(*l), span);
        }
    }
    /// Emits drops for scopes above `depth` without popping them (for brk/cont/ret).
    fn ret_inline(&mut self, v: &Option<Box<Expr>>, exit: BlockId, depth: usize, sp: Span) {
        if let Some(v) = v {
            let op = self.as_operand(v);
            if !v.ty.is_unit() {
                let dest = self.inline_dest.clone().unwrap();
                self.assign(&dest, Rvalue::Use(op), sp);
            }
        }
        self.drops_to_depth(depth, sp);
        self.terminate(Term::Goto(exit), sp);
        self.cur = self.new_block();
    }

    pub fn drops_to_depth(&mut self, depth: usize, span: Span) {
        for s in self.scopes[depth..].iter().rev() {
            for l in s.drops.iter().rev() {
                self.body.blocks[self.cur as usize].stmts.push((Stmt::Drop(Place::local(*l)), span));
            }
        }
    }
    fn local(&self, l: LocalId) -> LocalIdx {
        self.local_map[l.0 as usize]
    }

    pub fn is_copy(&self, t: &Ty) -> bool {
        is_copy(self.prog, t)
    }

    // ----- places and operands -----

    pub fn as_place(&mut self, e: &Expr) -> Place {
        match &e.kind {
            ExprKind::Local(l) => Place::local(self.local(*l)),
            ExprKind::Field { base, idx } => self.as_place(base).field(*idx),
            ExprKind::Deref(inner) => {
                if let ExprKind::Local(l) = &inner.kind {
                    return Place::local(self.local(*l)).deref();
                }
                let op = self.as_operand(inner);
                let t = self.temp(inner.ty.clone(), inner.span);
                self.push(Stmt::Assign(Place::local(t), Rvalue::Use(op)), inner.span);
                Place::local(t).deref()
            }
            _ => {
                let t = self.temp(e.ty.clone(), e.span);
                self.expr_into(e, Place::local(t), e.span);
                Place::local(t)
            }
        }
    }

    pub fn as_operand(&mut self, e: &Expr) -> Operand {
        match &e.kind {
            ExprKind::Lit(l) => Operand::Const(lit_const(l, &e.ty), e.ty.clone()),
            ExprKind::FnRef(d) => Operand::Const(Const::Fn(*d, Vec::new()), e.ty.clone()),
            ExprKind::Local(_) | ExprKind::Field { .. } | ExprKind::Deref(_) => {
                let p = self.as_place(e);
                if matches!(e.ty, Ty::Ref { mutable: true, .. }) {
                    return self.reborrow(p, &e.ty, false, e.span);
                }
                if self.is_copy(&e.ty) {
                    Operand::Copy(p)
                } else {
                    if !p.is_local() && !self.can_move_field(&p) {
                        let code = if p.has_deref() { "move.deref" } else { "move.field" };
                        self.diags.push(Diag::error(e.span, code).kv("fix", "(& ...) or mem.replace"));
                    }
                    Operand::Move(p)
                }
            }
            _ => {
                if e.ty.is_unit() {
                    self.expr_stmt(e);
                    return Operand::Const(Const::Unit, Ty::Unit);
                }
                let t = self.temp(e.ty.clone(), e.span);
                self.expr_into(e, Place::local(t), e.span);
                Operand::Move(Place::local(t))
            }
        }
    }

    /// Implicit reborrow: using a `&mut` place yields `&mut *place` instead of a move.
    pub fn reborrow(&mut self, p: Place, ty: &Ty, two_phase: bool, span: Span) -> Operand {
        let t = self.temp(ty.clone(), span);
        self.assign(&Place::local(t), Rvalue::Ref { mutable: true, place: p.deref(), two_phase }, span);
        Operand::Move(Place::local(t))
    }

    /// One field of a struct local that has no user `drop` method.
    fn can_move_field(&self, p: &Place) -> bool {
        if p.lone_field().is_none() {
            return false;
        }
        let ty = self.body.local_ty(p.local);
        if drop_method(self.prog, ty).is_some() {
            return false;
        }
        let Ty::Adt(d, _) = ty else { return false };
        matches!(self.prog.def(*d).kind, crate::resolve::DefKind::Struct(_))
    }

    /// Evaluates for effect only; drops any produced value.
    pub fn expr_stmt(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Let { local, init } => {
                let l = self.local(*local);
                let sc = self.scopes.last_mut().unwrap();
                sc.all.push(l);
                if let Some(init) = init {
                    if needs_drop(self.prog, &init.ty) {
                        sc.drops.push(l);
                    }
                    self.expr_into(init, Place::local(l), e.span);
                } else if needs_drop(self.prog, self.body.local_ty(l)) {
                    sc.drops.push(l);
                }
            }
            ExprKind::Assign { place, value } => {
                let v = self.as_operand(value);
                let p = self.as_place(place);
                if needs_drop(self.prog, &place.ty) && !self.through_ptr(place) {
                    self.push(Stmt::Drop(p.clone()), e.span);
                }
                self.push(Stmt::Assign(p, Rvalue::Use(v)), e.span);
            }
            ExprKind::Lit(_) | ExprKind::Local(_) => {}
            _ => {
                let t = self.temp(e.ty.clone(), e.span);
                self.expr_into(e, Place::local(t), e.span);
            }
        }
    }

    fn through_ptr(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Deref(inner) => matches!(inner.ty, Ty::Ptr(_)) || self.through_ptr(inner),
            ExprKind::Field { base, .. } => self.through_ptr(base),
            _ => false,
        }
    }

    pub fn assign(&mut self, dest: &Place, rv: Rvalue, span: Span) {
        self.push(Stmt::Assign(dest.clone(), rv), span);
    }

    // ----- expressions into a destination -----

    pub fn expr_into(&mut self, e: &Expr, dest: Place, span: Span) {
        let sp = e.span;
        match &e.kind {
            ExprKind::Lit(_) | ExprKind::FnRef(_) | ExprKind::Local(_) | ExprKind::Field { .. } | ExprKind::Deref(_) => {
                let op = self.as_operand(e);
                if !e.ty.is_unit() {
                    self.assign(&dest, Rvalue::Use(op), sp);
                }
            }
            ExprKind::Const(d) => {
                let c = match &self.prog.def(*d).kind {
                    crate::resolve::DefKind::Const(c) => c,
                    _ => unreachable!(),
                };
                let lit = match &c.value.kind {
                    crate::ast::ExprKind::Lit(l) => l.clone(),
                    _ => Lit::Unit,
                };
                self.assign(&dest, Rvalue::Use(Operand::Const(lit_const(&lit, &e.ty), e.ty.clone())), sp);
            }
            ExprKind::Let { .. } | ExprKind::Assign { .. } => self.expr_stmt(e),
            ExprKind::Block(items) => {
                self.push_scope();
                let n = items.len();
                for (i, it) in items.iter().enumerate() {
                    if i + 1 == n && !e.ty.is_unit() && !crate::check::is_stmt(it) {
                        self.expr_into(it, dest.clone(), span);
                    } else {
                        self.expr_stmt(it);
                    }
                }
                self.pop_scope(sp);
            }
            ExprKind::If { cond, then, els } => {
                let c = self.as_operand(cond);
                let tb = self.new_block();
                let eb = self.new_block();
                let join = self.new_block();
                self.terminate(Term::If(c, tb, eb), sp);
                self.cur = tb;
                if els.is_none() {
                    self.push_scope();
                    self.expr_stmt(then);
                    self.pop_scope(sp);
                } else {
                    self.expr_into(then, dest.clone(), span);
                }
                self.terminate(Term::Goto(join), sp);
                self.cur = eb;
                if let Some(els) = els {
                    self.expr_into(els, dest.clone(), span);
                }
                self.terminate(Term::Goto(join), sp);
                self.cur = join;
            }
            ExprKind::While { cond, body } => {
                let head = self.goto_new(sp);
                let bb = self.new_block();
                let exit = self.new_block();
                let c = self.as_operand(cond);
                self.terminate(Term::If(c, bb, exit), sp);
                self.cur = bb;
                self.loops.push(LoopCtx { brk: exit, cont: head, scope_depth: self.scopes.len() });
                self.expr_stmt(body);
                self.loops.pop();
                self.terminate(Term::Goto(head), sp);
                self.cur = exit;
            }
            ExprKind::Loop(body) => {
                let head = self.goto_new(sp);
                let exit = self.new_block();
                self.loops.push(LoopCtx { brk: exit, cont: head, scope_depth: self.scopes.len() });
                self.expr_stmt(body);
                self.loops.pop();
                self.terminate(Term::Goto(head), sp);
                self.cur = exit;
            }
            ExprKind::For { local, lo, hi, body } => {
                let i = self.local(*local);
                self.scopes.last_mut().unwrap().all.push(i);
                let lo_op = self.as_operand(lo);
                self.assign(&Place::local(i), Rvalue::Use(lo_op), sp);
                let hi_op = self.as_operand(hi);
                let hi_t = self.temp(hi.ty.clone(), sp);
                self.assign(&Place::local(hi_t), Rvalue::Use(hi_op), sp);
                let head = self.goto_new(sp);
                let bb = self.new_block();
                let step = self.new_block();
                let exit = self.new_block();
                let c = self.temp(Ty::Bool, sp);
                self.assign(&Place::local(c), Rvalue::Bin(BinOp::Lt, Operand::Copy(Place::local(i)), Operand::Copy(Place::local(hi_t))), sp);
                self.terminate(Term::If(Operand::Copy(Place::local(c)), bb, exit), sp);
                self.cur = bb;
                self.loops.push(LoopCtx { brk: exit, cont: step, scope_depth: self.scopes.len() });
                self.expr_stmt(body);
                self.loops.pop();
                self.terminate(Term::Goto(step), sp);
                self.cur = step;
                let one = Operand::Const(Const::Int(1, int_of(&lo.ty)), lo.ty.clone());
                self.assign(&Place::local(i), Rvalue::Bin(BinOp::Add, Operand::Copy(Place::local(i)), one), sp);
                self.terminate(Term::Goto(head), sp);
                self.cur = exit;
            }
            ExprKind::Break | ExprKind::Continue => {
                let lp = self.loops.last().expect("loop ctx");
                let (target, depth) = if matches!(e.kind, ExprKind::Break) { (lp.brk, lp.scope_depth) } else { (lp.cont, lp.scope_depth) };
                self.drops_to_depth(depth, sp);
                self.terminate(Term::Goto(target), sp);
                self.cur = self.new_block();
            }
            ExprKind::Return(v) => {
                if let Some((exit, depth)) = self.inline_exit {
                    self.ret_inline(v, exit, depth, sp);
                } else {
                    if let Some(v) = v {
                        let op = self.as_operand(v);
                        if !v.ty.is_unit() {
                            self.assign(&Place::local(0), Rvalue::Use(op), sp);
                        }
                    }
                    self.drops_to_depth(0, sp);
                    self.terminate(Term::Return, sp);
                    self.cur = self.new_block();
                }
            }
            ExprKind::Match { scrut, arms, by_ref } => self.lower_match(scrut, arms, *by_ref, dest, sp),
            ExprKind::Ref { mutable, place } => self.lower_ref(*mutable, place, &e.ty, dest, sp),
            ExprKind::Cast(inner) => {
                let op = self.as_operand(inner);
                self.assign(&dest, Rvalue::Cast(op, e.ty.clone()), sp);
            }
            ExprKind::Bin { op, l, r } => {
                if !self.raw_div && matches!(op, BinOp::Div | BinOp::Rem) && matches!(e.ty, Ty::Int(_)) {
                    self.lower_checked_div(*op, l, r, &dest, e.ty.clone(), sp);
                } else {
                    let a = self.as_operand(l);
                    let b = self.as_operand(r);
                    self.assign(&dest, Rvalue::Bin(*op, a, b), sp);
                }
            }
            ExprKind::Neg(x) | ExprKind::WNeg(x) => {
                let a = self.as_operand(x);
                let rv = if matches!(e.kind, ExprKind::WNeg(_)) { Rvalue::WNeg(a) } else { Rvalue::Neg(a) };
                self.assign(&dest, rv, sp);
            }
            ExprKind::Not(x) => {
                let a = self.as_operand(x);
                self.assign(&dest, Rvalue::Not(a), sp);
            }
            ExprKind::And(l, r) | ExprKind::Or(l, r) => {
                let is_and = matches!(e.kind, ExprKind::And(..));
                let a = self.as_operand(l);
                self.assign(&dest, Rvalue::Use(a), sp);
                let rb = self.new_block();
                let join = self.new_block();
                let (t, f) = if is_and { (rb, join) } else { (join, rb) };
                self.terminate(Term::If(Operand::Copy(dest.clone()), t, f), sp);
                self.cur = rb;
                let b = self.as_operand(r);
                self.assign(&dest, Rvalue::Use(b), sp);
                self.terminate(Term::Goto(join), sp);
                self.cur = join;
            }
            ExprKind::Call { callee, args } => self.lower_call(callee, args, &e.ty, dest, sp),
            ExprKind::DynPack { data, trait_id, self_ty } => self.lower_dyn_pack(data, *trait_id, self_ty, dest, sp),
            ExprKind::BoxPack { data, trait_id, self_ty } => self.lower_box_pack(data, *trait_id, self_ty, dest, sp),
            ExprKind::Struct { fields, .. } => {
                let ops: Vec<Operand> = fields.iter().map(|f| self.as_operand(f)).collect();
                self.assign(&dest, Rvalue::Struct(e.ty.clone(), ops), sp);
            }
            ExprKind::Variant { idx, fields, .. } => {
                let ops: Vec<Operand> = fields.iter().map(|f| self.as_operand(f)).collect();
                self.assign(&dest, Rvalue::Variant(e.ty.clone(), *idx, ops), sp);
            }
            ExprKind::Try { expr, is_res } => self.lower_try(expr, *is_res, dest, sp),
            ExprKind::Closure { caps, inner } => self.lower_clos(caps, inner, dest, sp),
        }
    }

}

pub(crate) fn const_static(prog: &Program, place: &Expr) -> Option<Const> {
    let ExprKind::Const(d) = &place.kind else { return None };
    let crate::resolve::DefKind::Const(c) = &prog.def(*d).kind else { return None };
    let crate::ast::ExprKind::Lit(l) = &c.value.kind else { return None };
    let k = lit_const(l, &c.ty);
    match k {
        Const::Int(..) | Const::Bool(_) | Const::Float(..) => Some(k),
        _ => None,
    }
}

pub fn lit_const(l: &Lit, ty: &Ty) -> Const {
    match l {
        Lit::Int(v, _) => match ty {
            Ty::Int(i) => Const::Int(*v, *i),
            Ty::F32 => Const::Float(*v as f64, true),
            Ty::F64 => Const::Float(*v as f64, false),
            _ => Const::Int(*v, IntTy::I64),
        },
        Lit::Float(v, _) => Const::Float(*v, matches!(ty, Ty::F32)),
        Lit::Bool(b) => Const::Bool(*b),
        Lit::Str(s) => Const::Str(s.clone()),
        Lit::Unit => Const::Unit,
    }
}

pub fn int_of(t: &Ty) -> IntTy {
    match t {
        Ty::Int(i) => *i,
        _ => IntTy::I64,
    }
}
