//! Match and pattern checking.
use super::Checker;
use crate::ast::{self, Lit};
use crate::diag::{Diag, PResult, Span};
use crate::thir::*;
use crate::ty::{DefId, Ty};

impl<'a> Checker<'a> {
    pub fn check_match(&mut self, scrut: &ast::Expr, arms: &[ast::Arm], want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        let scrut = self.check_expr(scrut, None)?;
        let st = self.shallow(&scrut.ty);
        let (inner, by_ref) = match &st {
            Ty::Ref { mutable, inner, .. } => (self.shallow(inner), Some(*mutable)),
            t => (t.clone(), None),
        };
        let enum_def = match &inner {
            Ty::Adt(d, _) if self.prog.enum_def(*d).is_some() => Some(*d),
            _ => None,
        };
        let mut out_arms = Vec::new();
        let mut result: Option<Ty> = want.cloned();
        let mut covered: Vec<u32> = Vec::new();
        let mut has_wild = false;
        let mut bools = [false, false];
        for arm in arms {
            self.push_scope();
            let pat = self.check_pat(&arm.pat, &inner, by_ref, enum_def);
            let pat = match pat {
                Ok(p) => p,
                Err(e) => {
                    self.pop_scope();
                    return Err(e);
                }
            };
            match &pat {
                Pat::Wild | Pat::Bind(_) => has_wild = true,
                Pat::Variant { idx, subs } if subs.iter().all(irrefutable) => covered.push(*idx),
                Pat::Lit(Lit::Bool(b)) => bools[*b as usize] = true,
                _ => {}
            }
            let body = self.check_expr(&arm.body, result.as_ref());
            self.pop_scope();
            let body = body?;
            let body = if super::is_stmt(&body) {
                let bsp = body.span;
                Expr { kind: ExprKind::Block(vec![body]), ty: Ty::Unit, span: bsp }
            } else {
                body
            };
            match &result {
                None => {
                    if !matches!(self.shallow(&body.ty), Ty::Never) {
                        result = Some(body.ty.clone());
                    }
                }
                Some(r) => {
                    let r = r.clone();
                    if !self.unify(&body.ty, &r) {
                        return Err(self.mismatch(body.span, &r, &body.ty));
                    }
                }
            }
            out_arms.push(Arm { pat, body, span: arm.pat.span() });
        }
        // exhaustiveness
        let exhaustive = has_wild
            || match (&inner, enum_def) {
                (_, Some(d)) => {
                    let n = self.prog.enum_def(d).unwrap().variants.len() as u32;
                    (0..n).all(|i| covered.contains(&i))
                }
                (Ty::Bool, _) => bools[0] && bools[1],
                _ => false,
            };
        if !exhaustive {
            let mut d = Diag::error(sp, "match.missing");
            if let Some(ed) = enum_def {
                let e = self.prog.enum_def(ed).unwrap();
                let missing: Vec<String> = e
                    .variants
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !covered.contains(&(*i as u32)))
                    .map(|(_, v)| format!("{}.{}", self.prog.type_name(ed), v.0))
                    .collect();
                d = d.kv("missing", missing.join(","));
            } else {
                d = d.kv("fix", "(_ ...)");
            }
            return Err(d);
        }
        let ty = result.unwrap_or(Ty::Never);
        Ok(Expr { kind: ExprKind::Match { scrut: Box::new(scrut), arms: out_arms, by_ref }, ty, span: sp })
    }

    fn check_pat(&mut self, p: &ast::Pat, ty: &Ty, by_ref: Option<bool>, enum_def: Option<DefId>) -> PResult<Pat> {
        match p {
            ast::Pat::Wild(_) => Ok(Pat::Wild),
            ast::Pat::Bind(name, sp) => {
                let bty = match by_ref {
                    Some(m) => Ty::Ref { mutable: m, lt: None, inner: Box::new(ty.clone()) },
                    None => ty.clone(),
                };
                let l = self.declare(name, bty, false, *sp)?;
                Ok(Pat::Bind(l))
            }
            ast::Pat::Lit(l, sp) => {
                let lit_ty = match l {
                    Lit::Int(_, _) => {
                        if !self.shallow(ty).is_int() {
                            return Err(self.mismatch(*sp, ty, &Ty::Int(crate::ty::IntTy::I64)));
                        }
                        ty.clone()
                    }
                    Lit::Bool(_) => Ty::Bool,
                    _ => return Err(Diag::error(*sp, "match.bad_lit")),
                };
                if !self.unify(&lit_ty, ty) {
                    return Err(self.mismatch(*sp, ty, &lit_ty));
                }
                Ok(Pat::Lit(l.clone()))
            }
            ast::Pat::Variant { path, subs, span } => {
                let d = enum_def.ok_or_else(|| Diag::error(*span, "match.not_enum").kv("got", self.show(ty)))?;
                let (tpath, vname) = path.rsplit_once('.').ok_or_else(|| Diag::error(*span, "type.no_variant").kv("variant", path.clone()))?;
                let pt = self.prog.lookup_type(self.module, tpath);
                self.require_visible(d, *span)?;
                if pt != Some(d) {
                    return Err(Diag::error(*span, "type.mismatch").kv("want", self.prog.type_name(d)).kv("got", tpath));
                }
                let ed = self.prog.enum_def(d).unwrap();
                let idx = ed.variants.iter().position(|v| v.0 == vname).ok_or_else(|| {
                    Diag::error(*span, "type.no_variant").kv("ty", self.prog.type_name(d)).kv("variant", vname)
                })?;
                self.require_variant(d, idx as u32, *span)?;
                let args = match ty {
                    Ty::Adt(_, a) => a.clone(),
                    _ => Vec::new(),
                };
                let ftys: Vec<Ty> = ed.variants[idx].1.iter().map(|t| t.subst(&args)).collect();
                if ftys.len() != subs.len() {
                    return Err(Diag::error(*span, "type.arity").kv("want", ftys.len().to_string()).kv("got", subs.len().to_string()));
                }
                let mut out = Vec::new();
                for (s, t) in subs.iter().zip(&ftys) {
                    let sub_enum = match self.shallow(t) {
                        Ty::Adt(d, _) if self.prog.enum_def(d).is_some() => Some(d),
                        _ => None,
                    };
                    out.push(self.check_pat(s, t, by_ref, sub_enum)?);
                }
                Ok(Pat::Variant { idx: idx as u32, subs: out })
            }
        }
    }
}

fn irrefutable(p: &Pat) -> bool {
    matches!(p, Pat::Wild | Pat::Bind(_))
}
