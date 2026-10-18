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
        let kill_from = self.body.locals.len() as LocalIdx;
        let saved = self.local_map.clone();
        self.install_locals(&body);
        self.bind_args(&body, args, sp);
        let sources = self.loan_sources(func, &body, dest);
        self.expr_into(&body.body, dest.clone(), sp);
        self.local_map = saved;
        self.push(Stmt::InlineLoans { kill_from, sources, dest: dest.local }, sp);
        self.inline_stack.pop();
        true
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
