//! Call checking: functions, methods, constructors, builtins.
use super::Checker;
use crate::ast::{self, TyExpr};
use crate::diag::{Diag, PResult, Span};
use crate::resolve::DefKind;
use crate::thir::*;
use crate::ty::{DefId, IntTy, Ty};

fn mk(kind: ExprKind, ty: Ty, span: Span) -> Expr {
    Expr { kind, ty, span }
}

impl<'a> Checker<'a> {
    pub fn check_call(&mut self, callee: &str, targs: &[TyExpr], args: &[ast::Expr], want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        let targs = targs.iter().map(|t| self.resolve_ty(t)).collect::<PResult<Vec<_>>>()?;
        // 1. local of fn type
        if let Some(l) = self.lookup_local(callee) {
            let ty = self.locals[l.0 as usize].ty.clone();
            let (ps, r) = match self.shallow(&ty) {
                Ty::Fn(ps, r) => (ps, *r),
                other => return Err(Diag::error(sp, "type.not_callable").kv("got", self.show(&other))),
            };
            let args = self.check_args(args, &ps, sp)?;
            let c = mk(ExprKind::Local(l), ty, sp);
            return Ok(mk(ExprKind::Call { callee: Callee::Ptr(Box::new(c)), args }, r, sp));
        }
        // 2. function or method
        if let Some(f) = self.prog.lookup_fn(self.module, callee) {
            self.require_visible(f, sp)?;
            return self.check_fn_call(f, &targs, args, want, sp);
        }
        // 3. struct constructor / enum variant
        if let Some(t) = self.prog.lookup_type(self.module, callee) {
            if self.prog.struct_def(t).is_some() {
                self.require_visible(t, sp)?;
                return self.check_struct_ctor(t, &targs, args, want, sp);
            }
        }
        if let Some((tpath, variant)) = callee.rsplit_once('.') {
            if let Some(t) = self.prog.lookup_type(self.module, tpath) {
                if let Some(ed) = self.prog.enum_def(t) {
                    self.require_visible(t, sp)?;
                    let idx = ed.variants.iter().position(|v| v.0 == variant).ok_or_else(|| {
                        Diag::error(sp, "type.no_variant").kv("ty", self.prog.type_name(t)).kv("variant", variant)
                    })?;
                    self.require_variant(t, idx as u32, sp)?;
                    return self.check_variant_ctor(t, idx as u32, &targs, args, want, sp);
                }
            }
        }
        // 4. builtins
        if let Some(b) = Builtin::from_path(callee) {
            return self.check_builtin(b, &targs, args, want, sp);
        }
        Err(Diag::error(sp, "name.unknown").kv("name", callee))
    }

    /// Instantiates `n` generics: explicit args when given, otherwise fresh variables.
    fn instantiate(&mut self, generics: &[String], explicit: &[Ty], sp: Span) -> PResult<Vec<Ty>> {
        if explicit.is_empty() {
            return Ok(generics.iter().map(|_| self.fresh(false)).collect());
        }
        if explicit.len() == generics.len() {
            return Ok(explicit.to_vec());
        }
        // Method with owner generics: explicit args may cover only the fn's own params.
        if explicit.len() < generics.len() {
            let mut out: Vec<Ty> = (0..generics.len() - explicit.len()).map(|_| self.fresh(false)).collect();
            out.extend(explicit.iter().cloned());
            return Ok(out);
        }
        Err(Diag::error(sp, "type.generic_arity").kv("want", generics.len().to_string()).kv("got", explicit.len().to_string()))
    }

    pub fn check_args(&mut self, args: &[ast::Expr], params: &[Ty], sp: Span) -> PResult<Vec<Expr>> {
        if args.len() != params.len() {
            return Err(Diag::error(sp, "type.arity").kv("want", params.len().to_string()).kv("got", args.len().to_string()));
        }
        let mut out = Vec::new();
        for (a, p) in args.iter().zip(params) {
            let e = self.check_expr(a, Some(p))?;
            self.expect(&e, p)?;
            out.push(e);
        }
        Ok(out)
    }

    fn check_fn_call(&mut self, f: DefId, explicit: &[Ty], args: &[ast::Expr], want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        let fd = self.prog.fn_def(f);
        let inst = self.instantiate(&fd.generics.clone(), explicit, sp)?;
        let params: Vec<Ty> = fd.params.iter().map(|p| p.1.subst(&inst)).collect();
        let ret = fd.ret.subst(&inst);
        if let Some(w) = want {
            // Propagate the expected type into generic results before checking args.
            let _ = self.unify(&ret, w);
        }
        let args = self.check_args(args, &params, sp)?;
        Ok(mk(ExprKind::Call { callee: Callee::Fn(f, inst), args }, ret, sp))
    }

    fn check_struct_ctor(&mut self, t: DefId, explicit: &[Ty], args: &[ast::Expr], want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        let sd = self.prog.struct_def(t).unwrap();
        let inst = self.instantiate(&sd.generics.clone(), explicit, sp)?;
        let ty = Ty::Adt(t, inst.clone());
        if let Some(w) = want {
            let _ = self.unify(&ty, w);
        }
        if self.prog.def(t).module != self.module {
            if let Some((fname, fty, _)) = sd.fields.iter().find(|f| !f.2) {
                let owner = self.prog.def(t).module;
                return Err(Diag::error(sp, "name.private_field")
                    .kv("field", fname.clone())
                    .kv("ty", self.prog.type_name(t))
                    .kv("module", self.prog.modules[owner as usize].path.clone())
                    .kv("fix", format!("(pub ({} {}))", fname, self.show(&fty.subst(&inst)))));
            }
        }
        let params: Vec<Ty> = sd.fields.iter().map(|f| f.1.subst(&inst)).collect();
        let fields = self.check_args(args, &params, sp)?;
        Ok(mk(ExprKind::Struct { def: t, fields }, ty, sp))
    }

    fn check_variant_ctor(&mut self, t: DefId, idx: u32, explicit: &[Ty], args: &[ast::Expr], want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        let ed = self.prog.enum_def(t).unwrap();
        let inst = self.instantiate(&ed.generics.clone(), explicit, sp)?;
        let ty = Ty::Adt(t, inst.clone());
        if let Some(w) = want {
            let _ = self.unify(&ty, w);
        }
        let params: Vec<Ty> = ed.variants[idx as usize].1.iter().map(|f| f.subst(&inst)).collect();
        let fields = self.check_args(args, &params, sp)?;
        Ok(mk(ExprKind::Variant { def: t, idx, fields }, ty, sp))
    }

    fn check_builtin(&mut self, b: Builtin, targs: &[Ty], args: &[ast::Expr], want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        let arity = |n: usize| -> PResult<()> {
            if args.len() != n {
                Err(Diag::error(sp, "type.arity").kv("want", n.to_string()).kv("got", args.len().to_string()))
            } else {
                Ok(())
            }
        };
        let (out, ty) = match b {
            Builtin::PtrNull => {
                arity(0)?;
                let t = match (targs.first(), want.map(|w| self.shallow(w))) {
                    (Some(t), _) => Ty::Ptr(Box::new(t.clone())),
                    (None, Some(Ty::Ptr(i))) => Ty::Ptr(i),
                    _ => Ty::Ptr(Box::new(self.fresh(false))),
                };
                (Vec::new(), t)
            }
            Builtin::PtrAdd => {
                arity(2)?;
                let p = self.check_expr(&args[0], None)?;
                let pt = self.shallow(&p.ty);
                if !matches!(pt, Ty::Ptr(_)) {
                    return Err(Diag::error(sp, "type.mismatch").kv("want", "(ptr T)").kv("got", self.show(&pt)));
                }
                let n = self.check_expr(&args[1], Some(&Ty::Int(IntTy::I64)))?;
                if !self.shallow(&n.ty).is_int() {
                    return Err(self.mismatch(n.span, &Ty::Int(IntTy::I64), &n.ty));
                }
                (vec![p, n], pt)
            }
            Builtin::PtrRead => {
                arity(1)?;
                let p = self.check_expr(&args[0], None)?;
                let inner = match self.shallow(&p.ty) {
                    Ty::Ptr(i) => *i,
                    other => return Err(Diag::error(sp, "type.mismatch").kv("want", "(ptr T)").kv("got", self.show(&other))),
                };
                (vec![p], inner)
            }
            Builtin::PtrWrite => {
                arity(2)?;
                let p = self.check_expr(&args[0], None)?;
                let inner = match self.shallow(&p.ty) {
                    Ty::Ptr(i) => *i,
                    other => return Err(Diag::error(sp, "type.mismatch").kv("want", "(ptr T)").kv("got", self.show(&other))),
                };
                let v = self.check_expr(&args[1], Some(&inner))?;
                self.expect(&v, &inner)?;
                (vec![p, v], Ty::Unit)
            }
            Builtin::PtrEq => {
                arity(2)?;
                let a = self.check_expr(&args[0], None)?;
                let b2 = self.check_expr(&args[1], Some(&a.ty.clone()))?;
                self.expect(&b2, &a.ty.clone())?;
                (vec![a, b2], Ty::Bool)
            }
            Builtin::MemReplace => {
                arity(2)?;
                let p = self.check_expr(&args[0], None)?;
                let inner = match self.shallow(&p.ty) {
                    Ty::Ref { mutable: true, inner, .. } => *inner,
                    other => return Err(Diag::error(sp, "type.mismatch").kv("want", "(&mut T)").kv("got", self.show(&other))),
                };
                let v = self.check_expr(&args[1], Some(&inner))?;
                self.expect(&v, &inner)?;
                (vec![p, v], inner)
            }
            Builtin::Forget => {
                arity(1)?;
                let v = self.check_expr(&args[0], None)?;
                (vec![v], Ty::Unit)
            }
            Builtin::Argc => {
                arity(0)?;
                (Vec::new(), Ty::Int(IntTy::I64))
            }
            Builtin::Argv => {
                arity(0)?;
                (Vec::new(), Ty::Ptr(Box::new(Ty::Ptr(Box::new(Ty::Int(IntTy::U8))))))
            }
            Builtin::Arch => {
                arity(0)?;
                (Vec::new(), self.ref_str())
            }
            Builtin::Drop => unreachable!(),
        };
        Ok(mk(ExprKind::Call { callee: Callee::Builtin(b), args: out }, ty, sp))
    }

    pub fn const_value(&self, d: DefId) -> &ast::Expr {
        match &self.prog.def(d).kind {
            DefKind::Const(c) => &c.value,
            _ => unreachable!(),
        }
    }
}
