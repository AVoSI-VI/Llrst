//! Lower a closure to a nested MIR function plus an env-building rvalue.
use super::lower::{lower_fn, Lower};
use super::*;
use crate::diag::Span;
use crate::thir::{ClosBody, Expr, FnBody};
use crate::ty::DefId;

impl<'a> Lower<'a> {
    pub fn lower_clos(&mut self, caps: &[Expr], inner: &ClosBody, dest: Place, sp: Span) {
        let ops: Vec<Operand> = caps.iter().map(|c| self.as_operand(c)).collect();
        let idx = self.body.closures.len() as u32;
        let name = format!("{}.clos{}", self.body.name, idx);
        let f = FnBody {
            def: DefId(u32::MAX),
            targs: Vec::new(),
            locals: inner.locals.clone(),
            params: inner.params.clone(),
            ret: inner.ret.clone(),
            body: (*inner.body).clone(),
            span: sp,
        };
        let nested = lower_fn(self.prog, self.diags, self.sm, &f, name, Vec::new(), self.checked);
        let mut nested = nested;
        let ncap = inner.caps.len();
        for blk in &mut nested.blocks {
            blk.stmts.retain(|(s, _)| !matches!(s, Stmt::Drop(p) if p.is_local() && (p.local as usize) >= 1 && (p.local as usize) <= ncap));
        }
        self.body.closures.push(ClosInst { body: nested, caps: inner.caps.clone(), user: inner.user.clone(), ret: inner.ret.clone() });
        self.assign(&dest, Rvalue::Closure { idx, caps: ops }, sp);
    }
}
