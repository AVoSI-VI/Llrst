//! `for` lowering. The no-step path is the historical `[lo, hi)` loop with step `+1`.
use super::lower::{int_of, LoopCtx, Lower};
use super::*;
use crate::diag::Span;
use crate::thir::{Expr, LocalId};

impl<'a> Lower<'a> {
    pub(crate) fn lower_for(&mut self, local: LocalId, lo: &Expr, hi: &Expr, step: &Option<Box<Expr>>, body: &Expr, sp: Span) {
        let i = self.local(local);
        self.scopes.last_mut().unwrap().all.push(i);
        let lo_op = self.as_operand(lo);
        self.assign(&Place::local(i), Rvalue::Use(lo_op), sp);
        let hi_op = self.as_operand(hi);
        let hi_t = self.temp(hi.ty.clone(), sp);
        self.assign(&Place::local(hi_t), Rvalue::Use(hi_op), sp);
        if let Some(step) = step {
            self.lower_for_step(i, hi_t, step, body, sp);
            return;
        }
        let head = self.goto_new(sp);
        let bb = self.new_block();
        let step_bb = self.new_block();
        let exit = self.new_block();
        let c = self.temp(Ty::Bool, sp);
        self.assign(&Place::local(c), Rvalue::Bin(BinOp::Lt, Operand::Copy(Place::local(i)), Operand::Copy(Place::local(hi_t))), sp);
        self.terminate(Term::If(Operand::Copy(Place::local(c)), bb, exit), sp);
        self.cur = bb;
        self.loops.push(LoopCtx { brk: exit, cont: step_bb, scope_depth: self.scopes.len() });
        self.expr_stmt(body);
        self.loops.pop();
        self.terminate(Term::Goto(step_bb), sp);
        self.cur = step_bb;
        let one = Operand::Const(Const::Int(1, int_of(&lo.ty)), lo.ty.clone());
        self.assign(&Place::local(i), Rvalue::Bin(BinOp::Add, Operand::Copy(Place::local(i)), one), sp);
        self.terminate(Term::Goto(head), sp);
        self.cur = exit;
    }

    /// Positive step tests `i < hi`. Negative step tests `i > hi`. Zero runs nothing.
    fn lower_for_step(&mut self, i: LocalIdx, hi_t: LocalIdx, step: &Expr, body: &Expr, sp: Span) {
        let st_op = self.as_operand(step);
        let st = self.temp(step.ty.clone(), sp);
        self.assign(&Place::local(st), Rvalue::Use(st_op), sp);
        let head = self.goto_new(sp);
        let bb = self.new_block();
        let step_bb = self.new_block();
        let exit = self.new_block();
        let pos_bb = self.new_block();
        let neg_bb = self.new_block();
        let chk = self.new_block();
        let zero = Operand::Const(Const::Int(0, int_of(&step.ty)), step.ty.clone());
        let pos = self.temp(Ty::Bool, sp);
        self.assign(&Place::local(pos), Rvalue::Bin(BinOp::Gt, Operand::Copy(Place::local(st)), zero.clone()), sp);
        self.terminate(Term::If(Operand::Copy(Place::local(pos)), pos_bb, chk), sp);
        self.cur = pos_bb;
        let c = self.temp(Ty::Bool, sp);
        self.assign(&Place::local(c), Rvalue::Bin(BinOp::Lt, Operand::Copy(Place::local(i)), Operand::Copy(Place::local(hi_t))), sp);
        self.terminate(Term::If(Operand::Copy(Place::local(c)), bb, exit), sp);
        self.cur = chk;
        let neg = self.temp(Ty::Bool, sp);
        self.assign(&Place::local(neg), Rvalue::Bin(BinOp::Lt, Operand::Copy(Place::local(st)), zero), sp);
        self.terminate(Term::If(Operand::Copy(Place::local(neg)), neg_bb, exit), sp);
        self.cur = neg_bb;
        let c2 = self.temp(Ty::Bool, sp);
        self.assign(&Place::local(c2), Rvalue::Bin(BinOp::Gt, Operand::Copy(Place::local(i)), Operand::Copy(Place::local(hi_t))), sp);
        self.terminate(Term::If(Operand::Copy(Place::local(c2)), bb, exit), sp);
        self.cur = bb;
        self.loops.push(LoopCtx { brk: exit, cont: step_bb, scope_depth: self.scopes.len() });
        self.expr_stmt(body);
        self.loops.pop();
        self.terminate(Term::Goto(step_bb), sp);
        self.cur = step_bb;
        self.assign(&Place::local(i), Rvalue::Bin(BinOp::Add, Operand::Copy(Place::local(i)), Operand::Copy(Place::local(st))), sp);
        self.terminate(Term::Goto(head), sp);
        self.cur = exit;
    }
}
