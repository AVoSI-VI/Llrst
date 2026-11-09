//! Match and pattern checking.
use super::Checker;
use crate::ast::{self, Lit};
use crate::diag::{Diag, PResult, Span};
use crate::thir::*;
use crate::ty::Ty;

struct Cov {
    full: bool,
    vars: Vec<u32>,
    bt: bool,
    bf: bool,
}

struct Re<'a> {
    names: &'a [String],
    seen: &'a mut Vec<String>,
}

impl<'a> Checker<'a> {
    pub fn check_match(&mut self, scrut: &ast::Expr, arms: &[ast::Arm], want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        let scrut = self.check_expr(scrut, None)?;
        let st = self.shallow(&scrut.ty);
        let (inner, by_ref) = match &st {
            Ty::Ref { mutable, inner, .. } => (self.shallow(inner), Some(*mutable)),
            t => (t.clone(), None),
        };
        let mut out_arms = Vec::new();
        let mut result: Option<Ty> = want.cloned();
        let mut cov = Cov::empty();
        for arm in arms {
            self.push_scope();
            let pat = match self.check_pat(&arm.pat, &inner, by_ref, None) {
                Ok(p) => p,
                Err(e) => {
                    self.pop_scope();
                    return Err(e);
                }
            };
            let guard = match arm.guard.as_ref() {
                Some(g) => match self.check_bool(g) {
                    Ok(e) => Some(e),
                    Err(e) => {
                        self.pop_scope();
                        return Err(e);
                    }
                },
                None => None,
            };
            if guard.is_none() {
                cov.join(self.cover(&pat, &inner));
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
            out_arms.push(Arm { pat, guard, body, span: arm.pat.span() });
        }
        if !self.type_full(&cov, &inner) {
            return Err(self.missing(sp, &inner, &cov));
        }
        let ty = result.unwrap_or(Ty::Never);
        Ok(Expr { kind: ExprKind::Match { scrut: Box::new(scrut), arms: out_arms, by_ref }, ty, span: sp })
    }

    fn missing(&self, sp: Span, ty: &Ty, cov: &Cov) -> Diag {
        let d = Diag::error(sp, "match.missing");
        if let Ty::Adt(id, _) = self.shallow(ty) {
            if let Some(e) = self.prog.enum_def(id) {
                let missing: Vec<String> = e
                    .variants
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !cov.vars.contains(&(*i as u32)))
                    .map(|(_, v)| format!("{}.{}", self.prog.type_name(id), v.0))
                    .collect();
                return d.kv("missing", missing.join(","));
            }
        }
        d.kv("fix", "(_ ...)")
    }

    fn check_pat(&mut self, p: &ast::Pat, ty: &Ty, by_ref: Option<bool>, mut re: Option<&mut Re<'_>>) -> PResult<Pat> {
        let ty = self.shallow(ty);
        match p {
            ast::Pat::Wild(_) => Ok(Pat::Wild),
            ast::Pat::Bind(name, sp) => self.check_bind(name, &ty, by_ref, *sp, re.as_deref_mut()),
            ast::Pat::Lit(l, sp) => self.pat_lit(l, &ty, *sp),
            ast::Pat::Or { subs, span } => self.check_or(subs, &ty, by_ref, *span, re.as_deref_mut()),
            ast::Pat::Variant { path, subs, span } => self.check_variant(path, subs, &ty, by_ref, *span, re.as_deref_mut()),
            ast::Pat::Struct { path, subs, span } => self.check_struct(path, subs, &ty, by_ref, *span, re.as_deref_mut()),
        }
    }

    fn check_bind(&mut self, name: &str, ty: &Ty, by_ref: Option<bool>, sp: Span, re: Option<&mut Re<'_>>) -> PResult<Pat> {
        let bty = bind_ty(ty, by_ref);
        if let Some(re) = re {
            if name == "_" {
                let l = self.declare(name, bty, false, sp)?;
                return Ok(Pat::Bind(l));
            }
            if re.seen.iter().any(|n| n == name) {
                return Err(Diag::error(sp, "name.dup").kv("name", name));
            }
            if !re.names.iter().any(|n| n == name) {
                return Err(Diag::error(sp, "match.or").kv("name", name));
            }
            let id = self.lookup_local(name).ok_or_else(|| Diag::error(sp, "match.or").kv("name", name))?;
            let have = self.zonk_ty(&self.locals[id.0 as usize].ty);
            let got = self.zonk_ty(&bty);
            if have != got && !self.unify(&have, &got) {
                return Err(self.mismatch(sp, &have, &got));
            }
            re.seen.push(name.to_string());
            return Ok(Pat::Bind(id));
        }
        Ok(Pat::Bind(self.declare(name, bty, false, sp)?))
    }

    fn pat_lit(&mut self, l: &Lit, ty: &Ty, sp: Span) -> PResult<Pat> {
        let lit_ty = match l {
            Lit::Int(_, _) => {
                if !ty.is_int() {
                    return Err(self.mismatch(sp, ty, &Ty::Int(crate::ty::IntTy::I64)));
                }
                ty.clone()
            }
            Lit::Bool(_) => Ty::Bool,
            Lit::Str(_) => {
                if !self.is_str(ty) {
                    return Err(self.mismatch(sp, ty, &self.ref_str()));
                }
                ty.clone()
            }
            _ => return Err(Diag::error(sp, "match.bad_lit")),
        };
        if !self.unify(&lit_ty, ty) {
            return Err(self.mismatch(sp, ty, &lit_ty));
        }
        Ok(Pat::Lit(l.clone()))
    }

    fn check_or(&mut self, subs: &[ast::Pat], ty: &Ty, by_ref: Option<bool>, sp: Span, re: Option<&mut Re<'_>>) -> PResult<Pat> {
        if subs.len() < 2 {
            return Err(Diag::error(sp, "parse.arity").kv("form", "or").kv("want", "2+").kv("got", subs.len().to_string()));
        }
        if let Some(re) = re {
            return self.or_re(subs, ty, by_ref, re);
        }
        let first = self.check_pat(&subs[0], ty, by_ref, None)?;
        let names = pat_names(&subs[0]);
        let mut out = vec![first];
        for s in &subs[1..] {
            let mut seen = Vec::new();
            let mut re = Re { names: &names, seen: &mut seen };
            out.push(self.check_pat(s, ty, by_ref, Some(&mut re))?);
            self.need_names(&names, &seen, s.span())?;
        }
        Ok(Pat::Or(out))
    }

    fn or_re(&mut self, subs: &[ast::Pat], ty: &Ty, by_ref: Option<bool>, re: &mut Re<'_>) -> PResult<Pat> {
        let mut local = Vec::new();
        for s in subs {
            walk_names(s, &mut local);
        }
        let mut out = Vec::new();
        for s in subs {
            let mut seen = Vec::new();
            let mut inner = Re { names: re.names, seen: &mut seen };
            out.push(self.check_pat(s, ty, by_ref, Some(&mut inner))?);
            self.need_names(&local, &seen, s.span())?;
        }
        for n in local {
            if !re.seen.iter().any(|s| s == &n) {
                re.seen.push(n);
            }
        }
        Ok(Pat::Or(out))
    }

    fn need_names(&self, names: &[String], seen: &[String], sp: Span) -> PResult<()> {
        for n in names {
            if !seen.iter().any(|s| s == n) {
                return Err(Diag::error(sp, "match.or").kv("name", n.clone()));
            }
        }
        Ok(())
    }

    fn check_variant(
        &mut self,
        path: &str,
        subs: &[ast::Pat],
        ty: &Ty,
        by_ref: Option<bool>,
        span: Span,
        mut re: Option<&mut Re<'_>>,
    ) -> PResult<Pat> {
        let d = match ty {
            Ty::Adt(d, _) if self.prog.enum_def(*d).is_some() => *d,
            _ => return Err(Diag::error(span, "match.not_enum").kv("got", self.show(ty))),
        };
        let (tpath, vname) = path.rsplit_once('.').ok_or_else(|| Diag::error(span, "type.no_variant").kv("variant", path))?;
        let pt = self.prog.lookup_type(self.module, tpath);
        self.require_visible(d, span)?;
        if pt != Some(d) {
            return Err(Diag::error(span, "type.mismatch").kv("want", self.prog.type_name(d)).kv("got", tpath));
        }
        let ed = self.prog.enum_def(d).unwrap();
        let idx = ed.variants.iter().position(|v| v.0 == vname).ok_or_else(|| {
            Diag::error(span, "type.no_variant").kv("ty", self.prog.type_name(d)).kv("variant", vname)
        })?;
        self.require_variant(d, idx as u32, span)?;
        let args = match ty {
            Ty::Adt(_, a) => a.clone(),
            _ => Vec::new(),
        };
        let ftys: Vec<Ty> = ed.variants[idx].1.iter().map(|t| t.subst(&args)).collect();
        if ftys.len() != subs.len() {
            return Err(Diag::error(span, "type.arity").kv("want", ftys.len().to_string()).kv("got", subs.len().to_string()));
        }
        let mut out = Vec::new();
        for (s, t) in subs.iter().zip(&ftys) {
            out.push(self.check_pat(s, t, by_ref, re.as_deref_mut())?);
        }
        Ok(Pat::Variant { idx: idx as u32, subs: out })
    }

    fn check_struct(
        &mut self,
        path: &str,
        subs: &[ast::Pat],
        ty: &Ty,
        by_ref: Option<bool>,
        span: Span,
        mut re: Option<&mut Re<'_>>,
    ) -> PResult<Pat> {
        let (d, args) = match ty {
            Ty::Adt(d, a) if self.prog.struct_def(*d).is_some() => (*d, a.clone()),
            _ => return Err(Diag::error(span, "type.mismatch").kv("want", path).kv("got", self.show(ty))),
        };
        self.require_visible(d, span)?;
        if self.prog.lookup_type(self.module, path) != Some(d) {
            return Err(Diag::error(span, "type.mismatch").kv("want", self.prog.type_name(d)).kv("got", path));
        }
        let sd = self.prog.struct_def(d).unwrap();
        if self.prog.def(d).module != self.module {
            if let Some((fname, fty, _)) = sd.fields.iter().find(|f| !f.2) {
                let owner = self.prog.def(d).module;
                return Err(Diag::error(span, "name.private_field")
                    .kv("field", fname.clone())
                    .kv("ty", self.prog.type_name(d))
                    .kv("module", self.prog.modules[owner as usize].path.clone())
                    .kv("fix", format!("(pub ({} {}))", fname, self.show(&fty.subst(&args)))));
            }
        }
        if sd.fields.len() != subs.len() {
            return Err(Diag::error(span, "type.arity").kv("want", sd.fields.len().to_string()).kv("got", subs.len().to_string()));
        }
        let mut out = Vec::new();
        for (s, f) in subs.iter().zip(&sd.fields) {
            out.push(self.check_pat(s, &f.1.subst(&args), by_ref, re.as_deref_mut())?);
        }
        Ok(Pat::Struct(out))
    }

    fn is_str(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Adt(d, _) => Some(*d) == self.prog.prelude_type("str"),
            _ => false,
        }
    }

    fn cover(&self, p: &Pat, ty: &Ty) -> Cov {
        let ty = self.shallow(ty);
        match p {
            Pat::Wild | Pat::Bind(_) => Cov::full(),
            Pat::Lit(Lit::Bool(true)) => Cov { bt: true, ..Cov::empty() },
            Pat::Lit(Lit::Bool(false)) => Cov { bf: true, ..Cov::empty() },
            Pat::Lit(_) => Cov::empty(),
            Pat::Variant { idx, subs } => self.cover_variant(*idx, subs, &ty),
            Pat::Struct(subs) => self.cover_struct(subs, &ty),
            Pat::Or(xs) => {
                let mut c = Cov::empty();
                for x in xs {
                    c.join(self.cover(x, &ty));
                }
                if self.type_full(&c, &ty) {
                    c.full = true;
                }
                c
            }
        }
    }

    fn cover_variant(&self, idx: u32, subs: &[Pat], ty: &Ty) -> Cov {
        let Ty::Adt(d, args) = ty else { return Cov::empty() };
        let Some(ed) = self.prog.enum_def(*d) else { return Cov::empty() };
        let ftys: Vec<Ty> = ed.variants[idx as usize].1.iter().map(|t| t.subst(args)).collect();
        if subs.len() != ftys.len() {
            return Cov::empty();
        }
        if subs.iter().zip(&ftys).all(|(s, t)| self.cover(s, t).full_for(self, t)) {
            let mut c = Cov::empty();
            c.vars.push(idx);
            c
        } else {
            Cov::empty()
        }
    }

    fn cover_struct(&self, subs: &[Pat], ty: &Ty) -> Cov {
        let Ty::Adt(d, args) = ty else { return Cov::empty() };
        let Some(sd) = self.prog.struct_def(*d) else { return Cov::empty() };
        if subs.len() != sd.fields.len() {
            return Cov::empty();
        }
        let ok = subs.iter().zip(&sd.fields).all(|(s, f)| {
            let ft = f.1.subst(args);
            self.cover(s, &ft).full_for(self, &ft)
        });
        if ok { Cov::full() } else { Cov::empty() }
    }

    fn type_full(&self, c: &Cov, ty: &Ty) -> bool {
        if c.full {
            return true;
        }
        match self.shallow(ty) {
            Ty::Bool => c.bt && c.bf,
            Ty::Adt(d, _) => self.prog.enum_def(d).is_some_and(|e| {
                (0..e.variants.len() as u32).all(|i| c.vars.contains(&i))
            }),
            _ => false,
        }
    }
}

impl Cov {
    fn empty() -> Cov {
        Cov { full: false, vars: Vec::new(), bt: false, bf: false }
    }
    fn full() -> Cov {
        Cov { full: true, vars: Vec::new(), bt: false, bf: false }
    }
    fn join(&mut self, o: Cov) {
        self.full |= o.full;
        self.bt |= o.bt;
        self.bf |= o.bf;
        for v in o.vars {
            if !self.vars.contains(&v) {
                self.vars.push(v);
            }
        }
    }
    fn full_for(&self, ck: &Checker, ty: &Ty) -> bool {
        self.full || ck.type_full(self, ty)
    }
}

fn bind_ty(ty: &Ty, by_ref: Option<bool>) -> Ty {
    match by_ref {
        Some(m) => Ty::Ref { mutable: m, lt: None, inner: Box::new(ty.clone()) },
        None => ty.clone(),
    }
}

fn pat_names(p: &ast::Pat) -> Vec<String> {
    let mut out = Vec::new();
    walk_names(p, &mut out);
    out
}

fn walk_names(p: &ast::Pat, out: &mut Vec<String>) {
    match p {
        ast::Pat::Bind(n, _) if n != "_" && !out.contains(n) => out.push(n.clone()),
        ast::Pat::Variant { subs, .. } | ast::Pat::Struct { subs, .. } | ast::Pat::Or { subs, .. } => {
            subs.iter().for_each(|s| walk_names(s, out));
        }
        _ => {}
    }
}
