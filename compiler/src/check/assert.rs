//! `(assert_eq ...)`, `(assert_ne ...)`, and `(assert_near ...)`.
use super::Checker;
use crate::ast::{self, Lit};
use crate::diag::{Diag, PResult, Span};
use crate::thir::*;
use crate::traitres::impl_of;
use crate::ty::{DefId, IntTy, Ty};

fn mk(kind: ExprKind, ty: Ty, span: Span) -> Expr {
    Expr { kind, ty, span }
}

impl<'a> Checker<'a> {
    pub fn check_assert_eq(&mut self, l: &ast::Expr, r: &ast::Expr, sp: Span, ne: bool) -> PResult<Expr> {
        let l = self.check_expr(l, None)?;
        let r = self.check_expr(r, Some(&l.ty.clone()))?;
        self.expect(&r, &l.ty.clone())?;
        let t = self.shallow(&l.ty);
        if let Some(built) = self.builtin_eq(&t, ne, sp)? {
            let (name, conv) = built;
            let f = self.test_fn(name, sp)?;
            let cast = |e: Expr| match &conv {
                Some(ty) => mk(ExprKind::Cast(Box::new(e)), ty.clone(), sp),
                None => e,
            };
            return Ok(self.unit_call(f, Vec::new(), vec![self.loc_lit(sp), cast(l), cast(r)], sp));
        }
        if matches!(t, Ty::F64 | Ty::F32) {
            self.require_show(&t, sp)?;
            if ne {
                return self.show_eq(l, r, &t, true, sp);
            }
            let conv = matches!(t, Ty::F32);
            let f = self.test_fn("eq_f64", sp)?;
            let cast = |e: Expr| {
                if conv { mk(ExprKind::Cast(Box::new(e)), Ty::F64, sp) } else { e }
            };
            let eps = mk(ExprKind::Lit(Lit::Float(0.0, None)), Ty::F64, sp);
            return Ok(self.unit_call(f, Vec::new(), vec![self.loc_lit(sp), cast(l), cast(r), eps], sp));
        }
        self.show_eq(l, r, &t, ne, sp)
    }

    pub fn check_assert_near(&mut self, g: &ast::Expr, w: &ast::Expr, eps: &ast::Expr, sp: Span) -> PResult<Expr> {
        let g = self.check_expr(g, None)?;
        let t = self.shallow(&g.ty);
        let conv = match &t {
            Ty::F64 => false,
            Ty::F32 => true,
            _ => return Err(Diag::error(sp, "type.assert_eq").kv("got", self.show(&t))),
        };
        let w = self.check_expr(w, Some(&g.ty))?;
        self.expect(&w, &g.ty)?;
        let eps = self.check_expr(eps, Some(&Ty::F64))?;
        self.expect(&eps, &Ty::F64)?;
        let f = self.test_fn("eq_f64", sp)?;
        let cast = |e: Expr| {
            if conv { mk(ExprKind::Cast(Box::new(e)), Ty::F64, sp) } else { e }
        };
        Ok(self.unit_call(f, Vec::new(), vec![self.loc_lit(sp), cast(g), cast(w), eps], sp))
    }

    /// `(& str)`, `bool`, `i64`, unsuffixed ints, and `u8` keep the original helpers.
    fn builtin_eq(&mut self, t: &Ty, ne: bool, _sp: Span) -> PResult<Option<(&'static str, Option<Ty>)>> {
        let intn = if ne { "ne_int" } else { "eq_int" };
        let out = match t {
            Ty::Ref { inner, .. } if self.zonk_ty(inner) == self.str_ty() => Some((if ne { "ne_str" } else { "eq_str" }, None)),
            Ty::Bool if !ne => Some(("eq_bool", None)),
            Ty::Int(IntTy::I64) => Some((intn, None)),
            Ty::Infer(_, true) => {
                self.unify(t, &Ty::Int(IntTy::I64));
                Some((intn, None))
            }
            Ty::Int(IntTy::U8) => Some((intn, Some(Ty::Int(IntTy::I64)))),
            _ => None,
        };
        Ok(out)
    }

    fn show_eq(&mut self, l: Expr, r: Expr, t: &Ty, ne: bool, sp: Span) -> PResult<Expr> {
        self.require_show(t, sp)?;
        let eq = self.trait_named("Eq").ok_or_else(|| Diag::error(sp, "name.unknown").kv("name", "Eq"))?;
        if impl_of(self.prog, eq, t).is_none() {
            return Err(Diag::error(sp, "trait.missing").kv("trait", "Eq").kv("ty", self.show(t)));
        }
        let name = if ne { "ne_by" } else { "eq_by" };
        let f = self.test_fn(name, sp)?;
        let args = vec![self.loc_lit(sp), shared(l), shared(r)];
        Ok(self.unit_call(f, vec![t.clone()], args, sp))
    }

    fn require_show(&self, t: &Ty, sp: Span) -> PResult<()> {
        let Some(id) = self.trait_named("Show") else {
            return Err(Diag::error(sp, "type.assert_eq").kv("got", self.show(t)));
        };
        if impl_of(self.prog, id, t).is_none() {
            return Err(Diag::error(sp, "type.assert_eq").kv("got", self.show(t)));
        }
        Ok(())
    }

    fn trait_named(&self, name: &str) -> Option<DefId> {
        self.prog.prelude_type(name).filter(|id| self.prog.trait_def(*id).is_some())
    }

    fn test_fn(&self, name: &str, sp: Span) -> PResult<DefId> {
        self.std_fn("test", name).ok_or_else(|| Diag::error(sp, "name.unknown").kv("name", format!("test.{name}")))
    }

    fn unit_call(&self, f: DefId, targs: Vec<Ty>, args: Vec<Expr>, sp: Span) -> Expr {
        mk(ExprKind::Call { callee: Callee::Fn(f, targs), args }, Ty::Unit, sp)
    }
}

fn shared(e: Expr) -> Expr {
    let sp = e.span;
    let ty = Ty::Ref { mutable: false, lt: None, inner: Box::new(e.ty.clone()) };
    mk(ExprKind::Ref { mutable: false, place: Box::new(e) }, ty, sp)
}
