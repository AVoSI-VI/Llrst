//! Substitute an `(inline (fn ...))` body into the caller.
use super::lower::Lower;
use super::*;
use crate::thir::FnBody;

impl<'a> Lower<'a> {
    /// Returns true when the call was replaced by the callee body.
    pub fn inline_call(&mut self, func: &Func, args: &[Operand], dest: &Place, sp: Span) -> bool {
        let Func::Def(d, targs) = func else { return false };
        if !self.prog.fn_def(*d).inline || self.inline_stack.contains(d) {
            return false;
        }
        self.inline_stack.push(*d);
        let body = match crate::check::check_fn(self.prog, self.diags, self.sm, *d, targs) {
            Ok(b) => b,
            Err(e) => {
                self.inline_stack.pop();
                self.diags.push(e);
                return false;
            }
        };
        self.subst(&body, func, args, dest, sp);
        self.inline_stack.pop();
        true
    }

    /// Substitutes `body` at the current block. `(ret ...)` jumps to a local exit.
    fn subst(&mut self, body: &FnBody, func: &Func, args: &[Operand], dest: &Place, sp: Span) {
        let kill_from = self.body.locals.len() as LocalIdx;
        let saved_map = self.local_map.clone();
        let saved_exit = self.inline_exit;
        let saved_dest = self.inline_dest.clone();
        let exit = self.new_block();
        let depth = self.scopes.len();
        self.push_scope();
        self.install_locals(body);
        self.bind_args(body, args, sp);
        self.keep_params(body);
        self.inline_exit = Some((exit, depth));
        self.inline_dest = Some(dest.clone());
        let sources = self.loan_sources(func, body, dest);
        self.expr_into(&body.body, dest.clone(), sp);
        self.pop_scope(sp);
        self.terminate(Term::Goto(exit), sp);
        self.cur = exit;
        self.inline_exit = saved_exit;
        self.inline_dest = saved_dest;
        self.local_map = saved_map;
        self.push(Stmt::InlineLoans { kill_from, sources, dest: dest.local }, sp);
    }

    fn keep_params(&mut self, body: &FnBody) {
        for p in &body.params {
            let l = self.local_map[p.0 as usize];
            if needs_drop(self.prog, self.body.local_ty(l)) {
                self.scopes.last_mut().unwrap().drops.push(l);
            }
        }
    }

    /// Parameter locals that hold the loans a call result would keep.
    fn loan_sources(&self, func: &Func, body: &FnBody, dest: &Place) -> Vec<LocalIdx> {
        if !place_ty(self.prog, &self.body, dest).has_ref() {
            return Vec::new();
        }
        let Func::Def(_, targs) = func else { return Vec::new() };
        let args: Vec<Operand> = body.params.iter().map(|p| Operand::Copy(Place::local(self.local_map[p.0 as usize]))).collect();
        crate::borrow::loans::eligible_args(self.prog, &Func::Def(body.def, targs.clone()), &args, &self.body)
            .into_iter()
            .filter_map(|i| body.params.get(i).map(|p| self.local_map[p.0 as usize]))
            .collect()
    }

    fn install_locals(&mut self, body: &FnBody) {
        self.local_map.clear();
        for loc in &body.locals {
            let idx = self.body.locals.len() as LocalIdx;
            self.body.locals.push(LocalDecl { ty: loc.ty.clone(), name: loc.name.clone(), mutable: loc.mutable, span: loc.span });
            self.local_map.push(idx);
        }
    }

    fn bind_args(&mut self, body: &FnBody, args: &[Operand], sp: Span) {
        for (p, op) in body.params.iter().zip(args) {
            let place = Place::local(self.local_map[p.0 as usize]);
            self.assign(&place, Rvalue::Use(op.clone()), sp);
        }
    }
}
