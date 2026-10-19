//! Type checker. Checks one function instance at a time against concrete type args and
//! produces THIR. Generic bodies are checked per instantiation.
mod call;
mod clos;
mod expr;
mod pat;
mod traits;
mod zonk;

use crate::ast;
use crate::diag::{Diag, Diags, PResult, SourceMap, Span};
use crate::resolve::{resolve_ty, DefKind, Program};
use crate::thir::*;
use crate::ty::{DefId, Ty, TyPrinter};
use std::collections::HashMap;

pub struct Checker<'a> {
    pub prog: &'a Program,
    pub diags: &'a Diags,
    pub module: u32,
    pub generics: Vec<String>,
    pub targs: Vec<Ty>,
    pub ret: Ty,
    pub locals: Vec<Local>,
    scopes: Vec<HashMap<String, LocalId>>,
    infer: Vec<Option<Ty>>,
    infer_int: Vec<bool>,
    loop_depth: u32,
    in_clos: bool,
    pub sm: &'a SourceMap,
}

pub fn check_fn<'a>(prog: &'a Program, diags: &'a Diags, sm: &'a SourceMap, def: DefId, targs: &[Ty]) -> PResult<FnBody> {
    let d = prog.def(def);
    let f = prog.fn_def(def);
    let mut c = Checker {
        prog,
        diags,
        module: d.module,
        generics: f.generics.clone(),
        targs: targs.to_vec(),
        ret: f.ret.subst(targs),
        locals: Vec::new(),
        scopes: vec![HashMap::new()],
        infer: Vec::new(),
        infer_int: Vec::new(),
        loop_depth: 0,
        in_clos: false,
        sm,
    };
    let mut params = Vec::new();
    for (name, ty) in &f.params {
        params.push(c.declare(name, ty.subst(targs), false, d.span)?);
    }
    let ret = c.ret.clone();
    let body = c.check_body(&f.body, &ret, d.span)?;
    let body = c.finish(body)?;
    Ok(FnBody { def, targs: targs.to_vec(), locals: c.locals, params, ret, body, span: d.span })
}

/// Checks a test body as a `fn () unit`.
pub fn check_test<'a>(prog: &'a Program, diags: &'a Diags, sm: &'a SourceMap, module: u32, t: &ast::TestDecl) -> PResult<FnBody> {
    let mut c = Checker {
        prog,
        diags,
        module,
        generics: Vec::new(),
        targs: Vec::new(),
        ret: Ty::Unit,
        locals: Vec::new(),
        scopes: vec![HashMap::new()],
        infer: Vec::new(),
        infer_int: Vec::new(),
        loop_depth: 0,
        in_clos: false,
        sm,
    };
    let body = c.check_body(&t.body, &Ty::Unit, t.span)?;
    let body = c.finish(body)?;
    Ok(FnBody { def: DefId(u32::MAX), targs: Vec::new(), locals: c.locals, params: Vec::new(), ret: Ty::Unit, body, span: t.span })
}

impl<'a> Checker<'a> {
    pub fn show(&self, t: &Ty) -> String {
        let t = self.zonk_ty(t);
        TyPrinter { names: &|d: DefId| self.prog.type_name(d) }.show(&t)
    }

    pub fn require_visible(&self, d: DefId, span: Span) -> PResult<()> {
        if self.prog.visible(self.module, d) {
            Ok(())
        } else {
            Err(self.prog.private_diag(span, d))
        }
    }

    pub fn mismatch(&self, span: Span, want: &Ty, got: &Ty) -> Diag {
        Diag::error(span, "type.mismatch").kv("want", self.show(want)).kv("got", self.show(got))
    }

    pub fn resolve_ty(&self, t: &ast::TyExpr) -> PResult<Ty> {
        Ok(resolve_ty(self.prog, self.module, &self.generics, t)?.subst(&self.targs))
    }

    // ----- inference variables -----

    pub fn fresh(&mut self, int: bool) -> Ty {
        self.infer.push(None);
        self.infer_int.push(int);
        Ty::Infer(self.infer.len() as u32 - 1, int)
    }

    pub fn shallow(&self, t: &Ty) -> Ty {
        let mut t = t.clone();
        while let Ty::Infer(i, _) = t {
            match &self.infer[i as usize] {
                Some(x) => t = x.clone(),
                None => break,
            }
        }
        t
    }

    pub fn zonk_ty(&self, t: &Ty) -> Ty {
        match self.shallow(t) {
            Ty::Ref { mutable, lt, inner } => Ty::Ref { mutable, lt, inner: Box::new(self.zonk_ty(&inner)) },
            Ty::Ptr(i) => Ty::Ptr(Box::new(self.zonk_ty(&i))),
            Ty::Adt(d, a) => Ty::Adt(d, a.iter().map(|x| self.zonk_ty(x)).collect()),
            Ty::Fn(ps, r) => Ty::Fn(ps.iter().map(|x| self.zonk_ty(x)).collect(), Box::new(self.zonk_ty(&r))),
            Ty::Clos(ps, r) => Ty::Clos(ps.iter().map(|x| self.zonk_ty(x)).collect(), Box::new(self.zonk_ty(&r))),
            t => t,
        }
    }

    /// Unifies two types. `Never` unifies with anything.
    pub fn unify(&mut self, a: &Ty, b: &Ty) -> bool {
        let a = self.shallow(a);
        let b = self.shallow(b);
        match (&a, &b) {
            (Ty::Never, _) | (_, Ty::Never) => true,
            (Ty::Infer(i, ai), Ty::Infer(j, bi)) => {
                if i == j {
                    return true;
                }
                // Keep the int constraint on the survivor.
                if *ai && !*bi {
                    self.infer[*j as usize] = Some(a.clone());
                    self.infer_int[*j as usize] = true;
                } else {
                    self.infer[*i as usize] = Some(b.clone());
                    if *ai {
                        self.infer_int[*j as usize] = true;
                    }
                }
                true
            }
            (Ty::Infer(i, int), other) | (other, Ty::Infer(i, int)) => {
                if *int && !matches!(other, Ty::Int(_)) {
                    return false;
                }
                self.infer[*i as usize] = Some(other.clone());
                true
            }
            (Ty::Ref { mutable: ma, inner: ia, .. }, Ty::Ref { mutable: mb, inner: ib, .. }) => ma == mb && self.unify(ia, ib),
            (Ty::Ptr(x), Ty::Ptr(y)) => self.unify(x, y),
            (Ty::Adt(d1, a1), Ty::Adt(d2, a2)) => d1 == d2 && a1.len() == a2.len() && a1.iter().zip(a2).all(|(x, y)| self.unify(x, y)),
            (Ty::Fn(p1, r1), Ty::Fn(p2, r2)) | (Ty::Clos(p1, r1), Ty::Clos(p2, r2)) => p1.len() == p2.len() && p1.iter().zip(p2).all(|(x, y)| self.unify(x, y)) && self.unify(r1, r2),
            _ => a == b,
        }
    }

    pub fn expect(&mut self, e: &Expr, want: &Ty) -> PResult<()> {
        if self.unify(&e.ty, want) {
            Ok(())
        } else {
            Err(self.mismatch(e.span, want, &e.ty))
        }
    }

    // ----- scopes -----

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn declare(&mut self, name: &str, ty: Ty, mutable: bool, span: Span) -> PResult<LocalId> {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(Local { name: name.to_string(), ty, mutable, span });
        if name != "_" {
            if self.scopes.last().unwrap().contains_key(name) {
                return Err(Diag::error(span, "name.dup").kv("name", name));
            }
            self.scopes.last_mut().unwrap().insert(name.to_string(), id);
        }
        Ok(id)
    }

    pub fn temp(&mut self, ty: Ty, span: Span) -> LocalId {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(Local { name: format!("$t{}", id.0), ty, mutable: true, span });
        id
    }

    pub fn lookup_local(&self, name: &str) -> Option<LocalId> {
        self.scopes.iter().rev().find_map(|s| s.get(name).copied())
    }

    pub fn in_loop(&self) -> bool {
        self.loop_depth > 0
    }
    pub fn enter_loop(&mut self) {
        self.loop_depth += 1;
    }
    pub fn exit_loop(&mut self) {
        self.loop_depth -= 1;
    }

    // ----- bodies -----

    /// Checks a sequence as a block whose value must have type `want`.
    pub fn check_body(&mut self, body: &[ast::Expr], want: &Ty, span: Span) -> PResult<Expr> {
        self.check_block(body, Some(want), span)
    }

    /// Checks a sequence as a block. With no expected type the block takes the type of its
    /// last value expression (unit when it ends in a statement).
    pub fn check_block(&mut self, body: &[ast::Expr], want: Option<&Ty>, span: Span) -> PResult<Expr> {
        self.push_scope();
        let mut out = Vec::new();
        let n = body.len();
        for (i, e) in body.iter().enumerate() {
            let last = i + 1 == n;
            let ex = if last { self.check_expr(e, want)? } else { self.check_expr(e, None)? };
            out.push(ex);
        }
        self.pop_scope();
        let ty = match out.last() {
            Some(e) if !is_stmt(e) => e.ty.clone(),
            Some(e) if matches!(e.ty, Ty::Never) => Ty::Never,
            _ => Ty::Unit,
        };
        let ty = match want {
            Some(w) => {
                if !self.unify(&ty, w) {
                    let sp = out.last().map_or(span, |e| e.span);
                    return Err(self.mismatch(sp, w, &ty));
                }
                w.clone()
            }
            None => ty,
        };
        Ok(Expr { kind: ExprKind::Block(out), ty, span })
    }

    pub fn std_fn(&self, module: &str, name: &str) -> Option<DefId> {
        let path = format!("std.{}", module);
        let m = self.prog.modules.iter().find(|m| m.path == path)?;
        m.items.get(name).copied().filter(|d| matches!(self.prog.def(*d).kind, DefKind::Fn(_)))
    }

    pub fn str_ty(&self) -> Ty {
        Ty::Adt(self.prog.prelude_type("str").expect("std str missing"), Vec::new())
    }

    pub fn ref_str(&self) -> Ty {
        Ty::Ref { mutable: false, lt: None, inner: Box::new(self.str_ty()) }
    }

    pub fn require_variant(&self, t: DefId, idx: u32, sp: Span) -> PResult<()> {
        let ed = self.prog.enum_def(t).unwrap();
        let (name, tys, is_pub) = &ed.variants[idx as usize];
        if *is_pub || self.prog.def(t).module == self.module {
            return Ok(());
        }
        let owner = self.prog.def(t).module;
        let mut fix = format!("(pub ({name}");
        for ty in tys {
            fix.push(' ');
            fix.push_str(&self.show(ty));
        }
        fix.push_str("))");
        Err(Diag::error(sp, "name.private_variant")
            .kv("variant", name.clone())
            .kv("ty", self.prog.type_name(t))
            .kv("module", self.prog.modules[owner as usize].path.clone())
            .kv("fix", fix))
    }

    /// Source location prefix for runtime diagnostics: `file|line|col`.
    pub fn loc_lit(&self, span: Span) -> Expr {
        let (line, col) = self.sm.line_col(span);
        let s = format!("{}|{}|{}", self.sm.path(span.file), line, col);
        Expr { kind: ExprKind::Lit(ast::Lit::Str(s.into_bytes())), ty: self.ref_str(), span }
    }

    fn finish(&mut self, body: Expr) -> PResult<Expr> {
        // Default unresolved int literals to i64.
        for i in 0..self.infer.len() {
            if self.infer[i].is_none() && self.infer_int[i] {
                self.infer[i] = Some(Ty::Int(crate::ty::IntTy::I64));
            }
        }
        zonk::zonk_body(self, body)
    }
}

/// Statement-like expressions have no value even when they are last in a block.
pub fn is_stmt(e: &Expr) -> bool {
    matches!(
        e.kind,
        ExprKind::Let { .. } | ExprKind::Assign { .. } | ExprKind::While { .. } | ExprKind::For { .. }
    ) || (matches!(e.kind, ExprKind::Loop(_)) && !matches!(e.ty, Ty::Never))
}
