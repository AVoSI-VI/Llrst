//! Expression checking.
use super::Checker;
use crate::ast::{self, BinOp, ExprKind as A, Lit};
use crate::diag::{Diag, PResult, Span};
use crate::resolve::DefKind;
use crate::thir::*;
use crate::ty::{IntTy, Ty};

fn mk(kind: ExprKind, ty: Ty, span: Span) -> Expr {
    Expr { kind, ty, span }
}

impl<'a> Checker<'a> {
    pub fn check_expr(&mut self, e: &ast::Expr, want: Option<&Ty>) -> PResult<Expr> {
        let sp = e.span;
        let out = match &e.kind {
            A::Lit(l) => self.check_lit(l, want, sp)?,
            A::Name(n) => self.check_name(n, sp)?,
            A::Let { name, mutable, ty, init } => {
                let declared = ty.as_ref().map(|t| self.resolve_ty(t)).transpose()?;
                let init = self.check_expr(init, declared.as_ref())?;
                let ty = match declared {
                    Some(t) => {
                        self.expect(&init, &t)?;
                        t
                    }
                    None => init.ty.clone(),
                };
                if matches!(self.shallow(&ty), Ty::Unit) {
                    return Err(Diag::error(sp, "type.unit_bind").kv("name", name.clone()));
                }
                let local = self.declare(name, ty, *mutable, sp)?;
                mk(ExprKind::Let { local, init: Box::new(init) }, Ty::Unit, sp)
            }
            A::Set { place, value } => {
                let place = self.check_expr(place, None)?;
                self.require_place(&place, true)?;
                let value = self.check_expr(value, Some(&place.ty.clone()))?;
                self.expect(&value, &place.ty.clone())?;
                mk(ExprKind::Assign { place: Box::new(place), value: Box::new(value) }, Ty::Unit, sp)
            }
            A::Do(items) => self.check_block(items, want, sp)?,
            A::If { cond, then, els } => {
                let cond = self.check_expr(cond, Some(&Ty::Bool))?;
                self.expect(&cond, &Ty::Bool)?;
                match els {
                    None => {
                        // The then-branch value is discarded; any type is accepted.
                        let then = self.check_branch(then, None)?;
                        mk(ExprKind::If { cond: Box::new(cond), then: Box::new(then), els: None }, Ty::Unit, sp)
                    }
                    Some(els) => {
                        let then = self.check_branch(then, want)?;
                        let els = self.check_branch(els, want.or(Some(&then.ty.clone())))?;
                        let ty = self.join(&then, &els)?;
                        mk(ExprKind::If { cond: Box::new(cond), then: Box::new(then), els: Some(Box::new(els)) }, ty, sp)
                    }
                }
            }
            A::While { cond, body } => {
                let cond = self.check_expr(cond, Some(&Ty::Bool))?;
                self.expect(&cond, &Ty::Bool)?;
                self.enter_loop();
                let body = self.check_body(body, &Ty::Unit, sp);
                self.exit_loop();
                mk(ExprKind::While { cond: Box::new(cond), body: Box::new(body?) }, Ty::Unit, sp)
            }
            A::Loop(body) => {
                self.enter_loop();
                let body = self.check_body(body, &Ty::Unit, sp);
                self.exit_loop();
                let body = body?;
                let ty = if has_break(&body) { Ty::Unit } else { Ty::Never };
                mk(ExprKind::Loop(Box::new(body)), ty, sp)
            }
            A::For { name, lo, hi, body } => {
                let lo = self.check_expr(lo, None)?;
                let hi = self.check_expr(hi, Some(&lo.ty.clone()))?;
                if !self.shallow(&lo.ty).is_int() {
                    return Err(Diag::error(lo.span, "type.not_numeric").kv("got", self.show(&lo.ty)));
                }
                self.expect(&hi, &lo.ty.clone())?;
                self.push_scope();
                let local = self.declare(name, lo.ty.clone(), false, sp)?;
                self.enter_loop();
                let body = self.check_body(body, &Ty::Unit, sp);
                self.exit_loop();
                self.pop_scope();
                mk(ExprKind::For { local, lo: Box::new(lo), hi: Box::new(hi), body: Box::new(body?) }, Ty::Unit, sp)
            }
            A::Break | A::Continue => {
                if !self.in_loop() {
                    return Err(Diag::error(sp, "flow.brk_outside"));
                }
                let k = if matches!(e.kind, A::Break) { ExprKind::Break } else { ExprKind::Continue };
                mk(k, Ty::Never, sp)
            }
            A::Return(v) => {
                let ret = self.ret.clone();
                let v = match v {
                    Some(v) => {
                        let v = self.check_expr(v, Some(&ret))?;
                        self.expect(&v, &ret)?;
                        Some(Box::new(v))
                    }
                    None => {
                        if !self.unify(&ret, &Ty::Unit) {
                            return Err(self.mismatch(sp, &ret, &Ty::Unit));
                        }
                        None
                    }
                };
                mk(ExprKind::Return(v), Ty::Never, sp)
            }
            A::Match { scrut, arms } => self.check_match(scrut, arms, want, sp)?,
            A::Ref { mutable, place } => {
                let place = self.check_expr(place, None)?;
                if place.is_place() {
                    self.require_place(&place, *mutable)?;
                }
                let ty = Ty::Ref { mutable: *mutable, lt: None, inner: Box::new(place.ty.clone()) };
                mk(ExprKind::Ref { mutable: *mutable, place: Box::new(place) }, ty, sp)
            }
            A::Deref(inner) => {
                let inner = self.check_expr(inner, None)?;
                let ty = match self.shallow(&inner.ty) {
                    Ty::Ref { inner: t, .. } | Ty::Ptr(t) => *t,
                    other => return Err(Diag::error(sp, "type.not_ref").kv("got", self.show(&other))),
                };
                mk(ExprKind::Deref(Box::new(inner)), ty, sp)
            }
            A::Field { base, name } => {
                let mut base = self.check_expr(base, None)?;
                // auto-deref through references
                loop {
                    match self.shallow(&base.ty) {
                        Ty::Ref { inner, .. } => {
                            let bsp = base.span;
                            base = mk(ExprKind::Deref(Box::new(base)), *inner, bsp);
                        }
                        _ => break,
                    }
                }
                let (idx, ty) = self.field_of(&base.ty, name, sp)?;
                mk(ExprKind::Field { base: Box::new(base), idx }, ty, sp)
            }
            A::Cast { ty, expr } => {
                let to = self.resolve_ty(ty)?;
                let inner = self.check_expr(expr, None)?;
                let from = self.shallow(&inner.ty);
                let ok = match (&from, &to) {
                    (Ty::Infer(_, true), t) if t.is_numeric() => {
                        self.unify(&from, &Ty::Int(IntTy::I64));
                        true
                    }
                    (f, t) if f.is_numeric() && t.is_numeric() => true,
                    (Ty::Bool, Ty::Int(_)) => true,
                    (Ty::Ptr(_), Ty::Ptr(_)) | (Ty::Ptr(_), Ty::Int(IntTy::U64)) | (Ty::Int(IntTy::U64), Ty::Ptr(_)) => true,
                    (Ty::Ref { .. }, Ty::Ptr(_)) => true,
                    _ => false,
                };
                if !ok {
                    return Err(Diag::error(sp, "type.bad_cast").kv("from", self.show(&from)).kv("to", self.show(&to)));
                }
                mk(ExprKind::Cast(Box::new(inner)), to, sp)
            }
            A::Bin { op, l, r } => self.check_bin(*op, l, r, want, sp)?,
            A::And(l, r) | A::Or(l, r) => {
                let l = self.check_expr(l, Some(&Ty::Bool))?;
                self.expect(&l, &Ty::Bool)?;
                let r = self.check_expr(r, Some(&Ty::Bool))?;
                self.expect(&r, &Ty::Bool)?;
                let k = if matches!(e.kind, A::And(..)) { ExprKind::And(Box::new(l), Box::new(r)) } else { ExprKind::Or(Box::new(l), Box::new(r)) };
                mk(k, Ty::Bool, sp)
            }
            A::Not(x) => {
                let x = self.check_expr(x, Some(&Ty::Bool))?;
                self.expect(&x, &Ty::Bool)?;
                mk(ExprKind::Not(Box::new(x)), Ty::Bool, sp)
            }
            A::WNeg(x) | A::Neg(x) => {
                let x = self.check_expr(x, want.filter(|t| t.is_numeric()))?;
                if !self.shallow(&x.ty).is_numeric() {
                    return Err(Diag::error(sp, "type.not_numeric").kv("got", self.show(&x.ty)));
                }
                let ty = x.ty.clone();
                let k = if matches!(e.kind, A::WNeg(_)) { ExprKind::WNeg(Box::new(x)) } else { ExprKind::Neg(Box::new(x)) };
                mk(k, ty, sp)
            }
            A::Call { callee, targs, args } => self.check_call(callee, targs, args, want, sp)?,
            A::CallPtr { callee, args } => {
                let callee = self.check_expr(callee, None)?;
                let (ps, r) = match self.shallow(&callee.ty) {
                    Ty::Fn(ps, r) => (ps, *r),
                    other => return Err(Diag::error(sp, "type.not_callable").kv("got", self.show(&other))),
                };
                let args = self.check_args(args, &ps, sp)?;
                mk(ExprKind::Call { callee: Callee::Ptr(Box::new(callee)), args }, r, sp)
            }
            A::Try(x) => self.check_try(x, sp)?,
            A::Panic(msg) => {
                let msg = self.check_expr(msg, Some(&self.ref_str()))?;
                self.expect(&msg, &self.ref_str())?;
                let f = self.std_fn("sys", "panic").ok_or_else(|| Diag::error(sp, "name.unknown").kv("name", "sys.panic"))?;
                let loc = self.loc_lit(sp);
                mk(ExprKind::Call { callee: Callee::Fn(f, Vec::new()), args: vec![loc, msg] }, Ty::Never, sp)
            }
            A::Assert(c) => {
                let c = self.check_expr(c, Some(&Ty::Bool))?;
                self.expect(&c, &Ty::Bool)?;
                let f = self.std_fn("test", "fail").ok_or_else(|| Diag::error(sp, "name.unknown").kv("name", "test.fail"))?;
                let loc = self.loc_lit(sp);
                let msg = mk(ExprKind::Lit(Lit::Str(b"assert".to_vec())), self.ref_str(), sp);
                let call = mk(ExprKind::Call { callee: Callee::Fn(f, Vec::new()), args: vec![loc, msg] }, Ty::Never, sp);
                let not = mk(ExprKind::Not(Box::new(c)), Ty::Bool, sp);
                mk(ExprKind::If { cond: Box::new(not), then: Box::new(call), els: None }, Ty::Unit, sp)
            }
            A::AssertEq(l, r) => self.check_assert_eq(l, r, sp)?,
            A::Sizeof(t) => {
                let t = self.resolve_ty(t)?;
                mk(ExprKind::Call { callee: Callee::Sizeof(t), args: Vec::new() }, Ty::Int(IntTy::U64), sp)
            }
            A::Drop(x) => {
                let x = self.check_expr(x, None)?;
                mk(ExprKind::Call { callee: Callee::Builtin(Builtin::Drop), args: vec![x] }, Ty::Unit, sp)
            }
        };
        match want {
            Some(w) => self.coerce(out, w),
            None => Ok(out),
        }
    }

    /// Unifies with the expected type, inserting the `&mut T` to `& T` coercion when needed.
    pub fn coerce(&mut self, e: Expr, want: &Ty) -> PResult<Expr> {
        if self.unify(&e.ty, want) {
            return Ok(e);
        }
        if let (Ty::Ref { mutable: false, inner: wi, .. }, Ty::Ref { mutable: true, inner: gi, .. }) = (self.shallow(want), self.shallow(&e.ty)) {
            if self.unify(&wi, &gi) {
                let sp = e.span;
                let d = mk(ExprKind::Deref(Box::new(e)), (*gi).clone(), sp);
                return Ok(mk(ExprKind::Ref { mutable: false, place: Box::new(d) }, Ty::Ref { mutable: false, lt: None, inner: wi }, sp));
            }
        }
        Err(self.mismatch(e.span, want, &e.ty))
    }

    /// A branch is a single expression, but `do` and statement forms yield unit.
    fn check_branch(&mut self, e: &ast::Expr, want: Option<&Ty>) -> PResult<Expr> {
        let ex = self.check_expr(e, want)?;
        if super::is_stmt(&ex) {
            let sp = ex.span;
            return Ok(mk(ExprKind::Block(vec![ex]), Ty::Unit, sp));
        }
        Ok(ex)
    }

    fn join(&mut self, a: &Expr, b: &Expr) -> PResult<Ty> {
        let ta = self.shallow(&a.ty);
        let tb = self.shallow(&b.ty);
        if matches!(ta, Ty::Never) {
            return Ok(tb);
        }
        if matches!(tb, Ty::Never) {
            return Ok(ta);
        }
        if !self.unify(&ta, &tb) {
            return Err(self.mismatch(b.span, &ta, &tb));
        }
        Ok(ta)
    }

    fn check_lit(&mut self, l: &Lit, want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        let ty = match l {
            Lit::Int(v, suffix) => {
                let t = match suffix {
                    Some(s) => Ty::Int(IntTy::from_name(s).ok_or_else(|| Diag::error(sp, "lex.bad_suffix").kv("got", s.clone()))?),
                    None => match want.map(|w| self.shallow(w)) {
                        Some(Ty::Int(i)) => Ty::Int(i),
                        Some(Ty::F32) => Ty::F32,
                        Some(Ty::F64) => Ty::F64,
                        _ => self.fresh(true),
                    },
                };
                if let Ty::Int(i) = &t {
                    if !i.fits(*v) {
                        return Err(Diag::error(sp, "lit.overflow").kv("ty", i.name()).kv("got", v.to_string()));
                    }
                }
                t
            }
            Lit::Float(_, suffix) => match suffix.as_deref() {
                Some("f32") => Ty::F32,
                Some("f64") | None => match want.map(|w| self.shallow(w)) {
                    Some(Ty::F32) if suffix.is_none() => Ty::F32,
                    _ => Ty::F64,
                },
                Some(s) => return Err(Diag::error(sp, "lex.bad_suffix").kv("got", s)),
            },
            Lit::Bool(_) => Ty::Bool,
            Lit::Str(_) => self.ref_str(),
            Lit::Unit => Ty::Unit,
        };
        Ok(mk(ExprKind::Lit(l.clone()), ty, sp))
    }

    fn check_name(&mut self, n: &str, sp: Span) -> PResult<Expr> {
        if let Some(l) = self.lookup_local(n) {
            let ty = self.locals[l.0 as usize].ty.clone();
            return Ok(mk(ExprKind::Local(l), ty, sp));
        }
        if let Some(c) = self.prog.lookup_const(self.module, n) {
            self.require_visible(c, sp)?;
            if let DefKind::Const(cd) = &self.prog.def(c).kind {
                return Ok(mk(ExprKind::Const(c), cd.ty.clone(), sp));
            }
        }
        if let Some(f) = self.prog.lookup_fn(self.module, n) {
            self.require_visible(f, sp)?;
            let fd = self.prog.fn_def(f);
            if !fd.generics.is_empty() {
                return Err(Diag::error(sp, "type.generic_fn_value").kv("name", n));
            }
            let ty = Ty::Fn(fd.params.iter().map(|p| p.1.clone()).collect(), Box::new(fd.ret.clone()));
            return Ok(mk(ExprKind::FnRef(f), ty, sp));
        }
        Err(Diag::error(sp, "name.unknown").kv("name", n))
    }

    pub fn field_of(&self, base: &Ty, name: &str, sp: Span) -> PResult<(u32, Ty)> {
        let base = self.shallow(base);
        if let Ty::Adt(d, args) = &base {
            if let Some(sd) = self.prog.struct_def(*d) {
                if let Some(i) = sd.fields.iter().position(|f| f.0 == name) {
                    return Ok((i as u32, sd.fields[i].1.subst(args)));
                }
            }
        }
        Err(Diag::error(sp, "type.no_field").kv("ty", self.show(&base)).kv("field", name))
    }

    /// Checks that `e` is a place and, when `write`, that it may be mutated.
    pub fn require_place(&self, e: &Expr, write: bool) -> PResult<()> {
        if !e.is_place() {
            return Err(Diag::error(e.span, "type.not_place"));
        }
        if !write {
            return Ok(());
        }
        let mut cur = e;
        loop {
            match &cur.kind {
                ExprKind::Local(l) => {
                    let loc = &self.locals[l.0 as usize];
                    if !loc.mutable {
                        return Err(Diag::error(e.span, "mut.required").kv("name", loc.name.clone()).kv("fix", format!("(let mut {} ...)", loc.name)));
                    }
                    return Ok(());
                }
                ExprKind::Field { base, .. } => cur = base,
                ExprKind::Deref(inner) => match self.shallow(&inner.ty) {
                    Ty::Ref { mutable: true, .. } | Ty::Ptr(_) => return Ok(()),
                    _ => return Err(Diag::error(e.span, "mut.required").kv("through", "&")),
                },
                _ => return Err(Diag::error(e.span, "type.not_place")),
            }
        }
    }

    fn check_bin(&mut self, op: BinOp, l: &ast::Expr, r: &ast::Expr, want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        let hint = if op.is_cmp() { None } else { want.filter(|t| t.is_numeric()) };
        let l = self.check_expr(l, hint)?;
        let r = self.check_expr(r, Some(&l.ty.clone()))?;
        self.expect(&r, &l.ty.clone())?;
        let lt = self.shallow(&l.ty);
        let ok = match op {
            BinOp::Eq | BinOp::Ne => lt.is_numeric() || matches!(lt, Ty::Bool | Ty::Ptr(_)),
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => lt.is_numeric(),
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => lt.is_int() || matches!(lt, Ty::Bool),
            BinOp::WAdd | BinOp::WSub | BinOp::WMul => lt.is_int(),
            _ => lt.is_numeric(),
        };
        if !ok {
            return Err(Diag::error(sp, "type.not_numeric").kv("op", format!("{:?}", op)).kv("got", self.show(&lt)));
        }
        let ty = if op.is_cmp() { Ty::Bool } else { l.ty.clone() };
        Ok(mk(ExprKind::Bin { op, l: Box::new(l), r: Box::new(r) }, ty, sp))
    }

    fn check_try(&mut self, x: &ast::Expr, sp: Span) -> PResult<Expr> {
        let inner = self.check_expr(x, None)?;
        let t = self.shallow(&inner.ty);
        let res = self.prog.prelude_type("res");
        let opt = self.prog.prelude_type("opt");
        let ret = self.shallow(&self.ret.clone());
        match (&t, &ret) {
            (Ty::Adt(d, a), Ty::Adt(rd, ra)) if Some(*d) == res && rd == d => {
                if !self.unify(&a[1], &ra[1]) {
                    return Err(self.mismatch(sp, &ra[1], &a[1]));
                }
                Ok(mk(ExprKind::Try { expr: Box::new(inner), is_res: true }, a[0].clone(), sp))
            }
            (Ty::Adt(d, a), Ty::Adt(rd, _)) if Some(*d) == opt && rd == d => {
                Ok(mk(ExprKind::Try { expr: Box::new(inner), is_res: false }, a[0].clone(), sp))
            }
            _ => Err(Diag::error(sp, "type.try").kv("got", self.show(&t)).kv("ret", self.show(&ret))),
        }
    }

    fn check_assert_eq(&mut self, l: &ast::Expr, r: &ast::Expr, sp: Span) -> PResult<Expr> {
        let l = self.check_expr(l, None)?;
        let r = self.check_expr(r, Some(&l.ty.clone()))?;
        self.expect(&r, &l.ty.clone())?;
        let t = self.shallow(&l.ty);
        let (name, conv): (&str, Option<Ty>) = match &t {
            Ty::Ref { inner, .. } if self.zonk_ty(inner) == self.str_ty() => ("eq_str", None),
            Ty::Bool => ("eq_bool", None),
            Ty::Int(IntTy::I64) => ("eq_int", None),
            Ty::Infer(_, true) => {
                self.unify(&t, &Ty::Int(IntTy::I64));
                ("eq_int", None)
            }
            Ty::Int(_) => ("eq_int", Some(Ty::Int(IntTy::I64))),
            _ => return Err(Diag::error(sp, "type.assert_eq").kv("got", self.show(&t))),
        };
        let f = self.std_fn("test", name).ok_or_else(|| Diag::error(sp, "name.unknown").kv("name", format!("test.{}", name)))?;
        let cast = |e: Expr| match &conv {
            Some(t) => mk(ExprKind::Cast(Box::new(e)), t.clone(), sp),
            None => e,
        };
        let loc = self.loc_lit(sp);
        Ok(mk(ExprKind::Call { callee: Callee::Fn(f, Vec::new()), args: vec![loc, cast(l), cast(r)] }, Ty::Unit, sp))
    }
}

/// True when a `brk` targets this loop body (not a nested loop).
fn has_break(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Break => true,
        ExprKind::While { .. } | ExprKind::Loop(_) | ExprKind::For { .. } => false,
        ExprKind::Block(items) => items.iter().any(has_break),
        ExprKind::Let { init, .. } => has_break(init),
        ExprKind::Assign { place, value } => has_break(place) || has_break(value),
        ExprKind::If { cond, then, els } => has_break(cond) || has_break(then) || els.as_ref().map_or(false, |e| has_break(e)),
        ExprKind::Match { scrut, arms, .. } => has_break(scrut) || arms.iter().any(|a| has_break(&a.body)),
        ExprKind::Call { args, .. } | ExprKind::Struct { fields: args, .. } | ExprKind::Variant { fields: args, .. } => args.iter().any(has_break),
        ExprKind::Ref { place: x, .. } | ExprKind::Deref(x) | ExprKind::Field { base: x, .. } | ExprKind::Cast(x) | ExprKind::Neg(x) | ExprKind::WNeg(x) | ExprKind::Not(x) => has_break(x),
        ExprKind::Bin { l, r, .. } | ExprKind::And(l, r) | ExprKind::Or(l, r) => has_break(l) || has_break(r),
        ExprKind::Return(Some(x)) | ExprKind::Try { expr: x, .. } => has_break(x),
        _ => false,
    }
}
