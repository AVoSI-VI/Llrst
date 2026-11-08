//! `(try e)` on `res` and `opt`. A different `res` error converts through `From`.
use super::Checker;
use crate::ast;
use crate::diag::{Diag, PResult, Span};
use crate::thir::*;
use crate::traitres::{impl_of, impl_subst};
use crate::ty::{DefId, Ty};

fn mk(kind: ExprKind, ty: Ty, span: Span) -> Expr {
    Expr { kind, ty, span }
}

impl<'a> Checker<'a> {
    pub fn check_try(&mut self, x: &ast::Expr, sp: Span) -> PResult<Expr> {
        let inner = self.check_expr(x, None)?;
        let t = self.shallow(&inner.ty);
        let res = self.prog.prelude_type("res");
        let opt = self.prog.prelude_type("opt");
        let ret = self.shallow(&self.ret.clone());
        match (&t, &ret) {
            (Ty::Adt(d, a), Ty::Adt(rd, ra)) if Some(*d) == res && rd == d => {
                let ok = a[0].clone();
                let from = if self.unify(&a[1], &ra[1]) {
                    None
                } else {
                    Some(self.from_fn(&ra[1], &a[1]).ok_or_else(|| self.mismatch(sp, &ra[1], &a[1]))?)
                };
                Ok(mk(ExprKind::Try { expr: Box::new(inner), is_res: true, from }, ok, sp))
            }
            (Ty::Adt(d, a), Ty::Adt(rd, _)) if Some(*d) == opt && rd == d => {
                Ok(mk(ExprKind::Try { expr: Box::new(inner), is_res: false, from: None }, a[0].clone(), sp))
            }
            _ => Err(Diag::error(sp, "type.try").kv("got", self.show(&t)).kv("ret", self.show(&ret))),
        }
    }

    /// `From.from` for `dest`, when its parameter is `src`.
    fn from_fn(&self, dest: &Ty, src: &Ty) -> Option<(DefId, Vec<Ty>)> {
        let tr = self.prog.prelude_type("From")?;
        let dest = self.zonk_ty(dest);
        let src = self.zonk_ty(src);
        let ids = impl_of(self.prog, tr, &dest)?;
        let f = *ids.first()?;
        let targs = impl_subst(self.prog, tr, &dest);
        let param = self.prog.fn_def(f).params.first()?.1.subst(&targs);
        if self.zonk_ty(&param) != src {
            return None;
        }
        Some((f, targs))
    }
}
