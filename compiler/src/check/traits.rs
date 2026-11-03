//! Trait calls, bounds, and `&T` to `& (dyn Trait)` packing.
use super::Checker;
use crate::diag::{Diag, PResult, Span};
use crate::thir::*;
use crate::traitres::{self, impl_of};
use crate::ty::{DefId, Ty};

fn mk(kind: ExprKind, ty: Ty, span: Span) -> Expr {
    Expr { kind, ty, span }
}

impl<'a> Checker<'a> {
    pub fn check_bounds(&self, bounds: &[Vec<DefId>], inst: &[Ty], sp: Span) -> PResult<()> {
        for (bs, ty) in bounds.iter().zip(inst) {
            let ty = self.zonk_ty(ty);
            for tr in bs {
                if impl_of(self.prog, *tr, &ty).is_none() {
                    return Err(Diag::error(sp, "trait.missing").kv("trait", self.prog.type_name(*tr)).kv("ty", self.show(&ty)));
                }
            }
        }
        Ok(())
    }

    pub fn trait_call(&mut self, tr: DefId, method: &str, args: &[crate::ast::Expr], sp: Span) -> PResult<Option<Expr>> {
        let Some(td) = self.prog.trait_def(tr) else { return Ok(None) };
        let slot = td.methods.iter().position(|(n, _, _)| n == method).ok_or_else(|| {
            Diag::error(sp, "trait.method").kv("name", method)
        })?;
        if args.is_empty() {
            return Err(Diag::error(sp, "type.arity").kv("want", td.methods[slot].1.len().to_string()).kv("got", "0"));
        }
        let recv = self.check_expr(&args[0], None)?;
        let got = self.shallow(&recv.ty);
        if let Ty::Dyn { trait_id, .. } = &got {
            if *trait_id != tr {
                return Err(self.mismatch(sp, &Ty::Dyn { mutable: false, trait_id: tr }, &got));
            }
            return self.dyn_call(tr, slot, recv, &args[1..], sp);
        }
        let self_ty = traitres::peel_self(&got).unwrap_or(got);
        let self_ty = self.force_int(self_ty);
        let self_ty = self.zonk_ty(&self_ty);
        let ids = impl_of(self.prog, tr, &self_ty).ok_or_else(|| {
            Diag::error(sp, "trait.missing").kv("trait", self.prog.type_name(tr)).kv("ty", self.show(&self_ty))
        })?;
        let f = ids[slot];
        let mut all = vec![args[0].clone()];
        all.extend(args[1..].iter().cloned());
        self.check_fn_call(f, &[], &all, None, sp).map(Some)
    }

    fn dyn_call(&mut self, tr: DefId, slot: usize, recv: Expr, rest: &[crate::ast::Expr], sp: Span) -> PResult<Option<Expr>> {
        let td = self.prog.trait_def(tr).unwrap();
        let fat = recv.ty.clone();
        let (params, ret) = traitres::dyn_sig(td, slot, &fat);
        if rest.len() + 1 != params.len() {
            return Err(Diag::error(sp, "type.arity").kv("want", params.len().to_string()).kv("got", (rest.len() + 1).to_string()));
        }
        let mut args = vec![recv];
        for (a, p) in rest.iter().zip(params.iter().skip(1)) {
            let e = self.check_expr(a, Some(p))?;
            args.push(self.finish_arg(e, p)?);
        }
        Ok(Some(mk(ExprKind::Call { callee: Callee::Dyn { trait_id: tr, slot: slot as u32 }, args }, ret, sp)))
    }

    fn force_int(&mut self, ty: Ty) -> Ty {
        if let Ty::Infer(_, true) = &ty {
            let _ = self.unify(&ty, &Ty::Int(crate::ty::IntTy::I64));
            return Ty::Int(crate::ty::IntTy::I64);
        }
        ty
    }

    pub fn finish_arg(&mut self, e: Expr, want: &Ty) -> PResult<Expr> {
        let got = self.shallow(&e.ty);
        let want_s = self.shallow(want);
        if let Ty::BoxDyn { trait_id } = &want_s {
            return self.pack_box(e, *trait_id, &want_s);
        }
        if let Ty::Dyn { trait_id, mutable } = want_s.clone() {
            if let Ty::Ref { mutable: m, inner, .. } = &got {
                if *m == mutable {
                    let self_ty = self.force_int(self.zonk_ty(inner));
                    if impl_of(self.prog, trait_id, &self_ty).is_none() {
                        return Err(Diag::error(e.span, "trait.missing").kv("trait", self.prog.type_name(trait_id)).kv("ty", self.show(&self_ty)));
                    }
                    let span = e.span;
                    return Ok(mk(ExprKind::DynPack { data: Box::new(e), trait_id, self_ty }, want_s, span));
                }
            }
            if self.unify(&got, &want_s) {
                return Ok(e);
            }
            return Err(self.mismatch(e.span, &want_s, &got));
        }
        if self.unify(&e.ty, want) {
            Ok(e)
        } else {
            Err(self.mismatch(e.span, want, &e.ty))
        }
    }

    /// `(& box)` is a dyn fat pointer, unless the expected type is a reference to the box.
    pub fn box_reborrow(&mut self, place_ty: &Ty, mutable: bool, want: Option<&Ty>) -> Option<DefId> {
        let Ty::BoxDyn { trait_id } = self.shallow(place_ty) else { return None };
        let Some(w) = want else { return Some(trait_id) };
        match self.shallow(w) {
            Ty::Ref { inner, .. } if matches!(self.shallow(inner.as_ref()), Ty::BoxDyn { .. }) => None,
            Ty::Dyn { trait_id: t, mutable: m } if m == mutable && t == trait_id => Some(trait_id),
            _ => None,
        }
    }

    fn pack_box(&mut self, e: Expr, trait_id: DefId, want: &Ty) -> PResult<Expr> {
        let got = self.shallow(&e.ty);
        if self.unify(&got, want) {
            return Ok(e);
        }
        let self_ty = self.force_int(self.zonk_ty(&got));
        if impl_of(self.prog, trait_id, &self_ty).is_none() {
            return Err(Diag::error(e.span, "trait.missing").kv("trait", self.prog.type_name(trait_id)).kv("ty", self.show(&self_ty)));
        }
        let span = e.span;
        Ok(mk(ExprKind::BoxPack { data: Box::new(e), trait_id, self_ty }, want.clone(), span))
    }
}
