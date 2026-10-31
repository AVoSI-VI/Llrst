//! Trait definitions, impls, and `(& (dyn Trait))`.
use crate::ast::{self, Item};
use crate::diag::{Diag, Diags, PResult, Span};
use crate::resolve::{resolve_ty, Def, DefKind, Program, TraitDef};
use crate::ty::{DefId, Ty};

/// `(box (dyn Trait))`. The single argument must be `(dyn Trait)`.
pub fn box_ty(prog: &Program, module: u32, args: &[ast::TyExpr], sp: Span) -> PResult<Ty> {
    let Some(ast::TyExpr::Dyn(name, _)) = args.first() else {
        return Err(Diag::error(sp, "trait.object").kv("why", "dyn"));
    };
    if args.len() != 1 {
        return Err(Diag::error(sp, "trait.object").kv("why", "dyn"));
    }
    let Ty::Dyn { trait_id, .. } = dyn_ty(prog, module, false, name, sp)? else { unreachable!() };
    Ok(Ty::BoxDyn { trait_id })
}

pub fn dyn_ty(prog: &Program, module: u32, mutable: bool, name: &str, sp: Span) -> PResult<Ty> {
    let id = lookup(prog, module, name).ok_or_else(|| Diag::error(sp, "name.unknown").kv("name", name))?;
    let tr = prog.trait_def(id).ok_or_else(|| Diag::error(sp, "name.not_type").kv("name", name))?;
    if !tr.object_safe {
        return Err(Diag::error(sp, "trait.object").kv("why", "self"));
    }
    if !prog.visible(module, id) {
        return Err(prog.private_diag(sp, id));
    }
    Ok(Ty::Dyn { mutable, trait_id: id })
}

pub fn fn_generics(prog: &Program, module: u32, outer: &[String], g: &ast::Generics, sp: Span) -> PResult<(Vec<String>, Vec<Vec<DefId>>)> {
    let mut names = outer.to_vec();
    let mut bounds = vec![Vec::new(); outer.len()];
    for (name, bs) in g.types.iter().zip(g.bounds.iter()) {
        let ids = trait_ids(prog, module, bs, sp)?;
        if let Some(i) = outer.iter().position(|o| o == name) {
            bounds[i] = ids;
        } else {
            names.push(name.clone());
            bounds.push(ids);
        }
    }
    Ok((names, bounds))
}

pub fn load(prog: &mut Program, parsed: &[(String, u32, Vec<Item>, bool)], diags: &Diags) -> PResult<()> {
    let mut traits = Vec::new();
    let mut impls = Vec::new();
    for (mi, (_, _, items, is_std)) in parsed.iter().enumerate() {
        for it in items {
            match it {
                Item::Trait(t) => traits.push((mi as u32, t)),
                Item::TraitImpl(t) => impls.push((mi as u32, *is_std, t)),
                _ => {}
            }
        }
    }
    for (mi, t) in traits {
        fill_trait(prog, mi, t, diags)?;
    }
    impls.sort_by_key(|(mi, is_std, _)| (!is_std, *mi));
    for (mi, _, t) in impls {
        add_impl(prog, mi, t, diags)?;
    }
    Ok(())
}

fn fill_trait(prog: &mut Program, mi: u32, t: &ast::TraitDecl, diags: &Diags) -> PResult<()> {
    let id = prog.modules[mi as usize].items[&t.name];
    let mut methods = Vec::new();
    let mut safe = true;
    for m in &t.methods {
        let params = m.params.iter().map(|p| resolve_ty(prog, mi, &[], &p.ty)).collect::<PResult<Vec<_>>>()?;
        let ret = resolve_ty(prog, mi, &[], &m.ret)?;
        if params.first().map_or(true, |p| !is_ref_self(p)) {
            diags.push(Diag::error(m.span, "trait.method").kv("name", m.name.clone()));
        }
        if params.iter().any(self_by_value) || self_by_value(&ret) {
            safe = false;
        }
        methods.push((m.name.clone(), params, ret));
    }
    if let DefKind::Trait(td) = &mut prog.defs[id.0 as usize].kind {
        td.methods = methods;
        td.object_safe = safe;
    }
    Ok(())
}

fn add_impl(prog: &mut Program, mi: u32, t: &ast::TraitImpl, diags: &Diags) -> PResult<()> {
    let tr = lookup(prog, mi, &t.trait_name).ok_or_else(|| Diag::error(t.span, "name.unknown").kv("name", t.trait_name.clone()))?;
    let td = prog.trait_def(tr).ok_or_else(|| Diag::error(t.span, "name.not_type").kv("name", t.trait_name.clone()))?;
    let self_ty = resolve_ty(prog, mi, &[], &ast::TyExpr::Name(t.ty_name.clone(), t.span))?;
    let sigs = td.methods.clone();
    if t.methods.len() != sigs.len() {
        return Err(Diag::error(t.span, "trait.method").kv("name", t.trait_name.clone()));
    }
    let mut ids = Vec::new();
    for (m, (name, params, ret)) in t.methods.iter().zip(sigs.iter()) {
        if m.name != *name {
            return Err(Diag::error(m.span, "trait.method").kv("name", m.name.clone()));
        }
        let mut def = crate::resolve::finish_fn(prog, mi, m, &[], None, diags)?;
        for p in &mut def.params {
            p.1 = subst_self(&p.1, &self_ty);
        }
        def.ret = subst_self(&def.ret, &self_ty);
        let want_ps: Vec<Ty> = params.iter().map(|p| subst_self(p, &self_ty)).collect();
        let want_ret = subst_self(ret, &self_ty);
        if def.params.iter().map(|p| &p.1).ne(want_ps.iter()) || def.ret != want_ret {
            return Err(Diag::error(m.span, "type.mismatch").kv("want", t.trait_name.clone()).kv("got", m.name.clone()));
        }
        def.owner = Some(tr);
        let id = DefId(prog.defs.len() as u32);
        prog.defs.push(Def {
            name: format!("{}_{}", m.name, t.ty_name),
            module: mi,
            span: m.span,
            kind: DefKind::Fn(def),
            is_pub: true,
        });
        ids.push(id);
    }
    if prog.impls.insert((tr, self_ty), ids).is_some() {
        diags.push(Diag::error(t.span, "trait.dup").kv("trait", t.trait_name.clone()).kv("ty", t.ty_name.clone()));
    }
    Ok(())
}

pub fn subst_self(t: &Ty, self_ty: &Ty) -> Ty {
    match t {
        Ty::SelfTy => self_ty.clone(),
        Ty::Ref { mutable, lt, inner } => Ty::Ref { mutable: *mutable, lt: lt.clone(), inner: Box::new(subst_self(inner, self_ty)) },
        Ty::Ptr(inner) => Ty::Ptr(Box::new(subst_self(inner, self_ty))),
        Ty::Adt(d, a) => Ty::Adt(*d, a.iter().map(|x| subst_self(x, self_ty)).collect()),
        Ty::Fn(ps, r) => Ty::Fn(ps.iter().map(|p| subst_self(p, self_ty)).collect(), Box::new(subst_self(r, self_ty))),
        Ty::Clos(ps, r) => Ty::Clos(ps.iter().map(|p| subst_self(p, self_ty)).collect(), Box::new(subst_self(r, self_ty))),
        other => other.clone(),
    }
}

pub fn self_by_value(t: &Ty) -> bool {
    match t {
        Ty::SelfTy => true,
        Ty::Ref { .. } | Ty::Ptr(_) => false,
        Ty::Adt(_, a) => a.iter().any(self_by_value),
        Ty::Fn(ps, r) | Ty::Clos(ps, r) => ps.iter().any(self_by_value) || self_by_value(r),
        _ => false,
    }
}

fn is_ref_self(t: &Ty) -> bool {
    matches!(t, Ty::Ref { inner, .. } if matches!(inner.as_ref(), Ty::SelfTy))
}

fn trait_ids(prog: &Program, module: u32, names: &[String], sp: Span) -> PResult<Vec<DefId>> {
    names.iter().map(|n| lookup(prog, module, n).filter(|id| prog.trait_def(*id).is_some()).ok_or_else(|| Diag::error(sp, "name.unknown").kv("name", n.clone()))).collect()
}

pub fn lookup(prog: &Program, module: u32, name: &str) -> Option<DefId> {
    let m = &prog.modules[module as usize];
    if let Some(id) = m.items.get(name) {
        return Some(*id);
    }
    prog.prelude.get(name).copied()
}

pub fn peel_self(t: &Ty) -> Option<Ty> {
    match t {
        Ty::Ref { inner, .. } => Some((**inner).clone()),
        Ty::Dyn { .. } => None,
        other => Some(other.clone()),
    }
}

pub fn abi_sig(tr: &TraitDef, slot: usize) -> (Vec<Ty>, Ty) {
    let ptr = Ty::Ptr(Box::new(Ty::Int(crate::ty::IntTy::U8)));
    let (_, ps, ret) = &tr.methods[slot];
    let params = ps.iter().map(|p| if is_ref_self(p) { ptr.clone() } else { p.clone() }).collect();
    (params, ret.clone())
}
pub fn dyn_sig(tr: &TraitDef, slot: usize, fat: &Ty) -> (Vec<Ty>, Ty) {
    let (_, ps, ret) = &tr.methods[slot];
    let params = ps.iter().map(|p| if is_ref_self(p) { fat.clone() } else { subst_self(p, fat) }).collect();
    (params, subst_self(ret, fat))
}

pub fn impl_of<'p>(prog: &'p Program, tr: DefId, ty: &Ty) -> Option<&'p Vec<DefId>> {
    prog.impls.get(&(tr, ty.clone()))
}
