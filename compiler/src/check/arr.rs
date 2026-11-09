//! `(arr T N)` and the `arr.*` desugars. They never remain a `Callee::Builtin`.
use super::Checker;
use crate::ast::{self, BinOp, Lit};
use crate::diag::{Diag, PResult, Span};
use crate::thir::*;
use crate::ty::{IntTy, Ty};

fn mk(kind: ExprKind, ty: Ty, span: Span) -> Expr {
    Expr { kind, ty, span }
}

fn i64_ty() -> Ty {
    Ty::Int(IntTy::I64)
}

fn int_lit(v: i128, sp: Span) -> Expr {
    mk(ExprKind::Lit(Lit::Int(v, None)), i64_ty(), sp)
}

fn loc(id: LocalId, ty: Ty, sp: Span) -> Expr {
    mk(ExprKind::Local(id), ty, sp)
}

fn arr_ty(elem: Ty, n: u64) -> Ty {
    Ty::Arr(Box::new(elem), n)
}

impl<'a> Checker<'a> {
    pub fn check_arr(&mut self, b: Builtin, targs: &[Ty], args: &[ast::Expr], want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        match b {
            Builtin::ArrOf => self.arr_of(args, want, sp),
            Builtin::ArrFilled => self.arr_filled(targs, args, want, sp),
            Builtin::ArrLen => self.arr_len(args, sp),
            Builtin::ArrGet => self.arr_get(args, false, sp),
            Builtin::ArrGetMut => self.arr_get(args, true, sp),
            Builtin::ArrSet => self.arr_set(args, sp),
            _ => unreachable!(),
        }
    }

    fn arity(&self, args: &[ast::Expr], n: usize, sp: Span) -> PResult<()> {
        if args.len() == n {
            Ok(())
        } else {
            Err(Diag::error(sp, "type.arity").kv("want", n.to_string()).kv("got", args.len().to_string()))
        }
    }

    fn as_arr(&self, t: &Ty) -> Option<(Ty, u64)> {
        match self.shallow(t) {
            Ty::Arr(e, n) => Some((*e, n)),
            _ => None,
        }
    }

    fn arr_of(&mut self, args: &[ast::Expr], want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        if args.is_empty() {
            return Err(Diag::error(sp, "type.arity").kv("want", "1").kv("got", "0"));
        }
        let shaped = want.and_then(|w| self.as_arr(w));
        let first = self.check_expr(&args[0], shaped.as_ref().map(|(t, _)| t))?;
        let elem = match &shaped {
            Some((t, _)) => {
                self.expect(&first, t)?;
                t.clone()
            }
            None => first.ty.clone(),
        };
        let mut elems = vec![first];
        for a in &args[1..] {
            let e = self.check_expr(a, Some(&elem))?;
            self.expect(&e, &elem)?;
            elems.push(e);
        }
        let n = elems.len() as u64;
        if let Some((_, wn)) = shaped {
            if wn != n {
                return Err(self.mismatch(sp, &arr_ty(elem.clone(), wn), &arr_ty(elem, n)));
            }
        }
        Ok(mk(ExprKind::Arr(elems), arr_ty(elem, n), sp))
    }

    fn arr_filled(&mut self, targs: &[Ty], args: &[ast::Expr], want: Option<&Ty>, sp: Span) -> PResult<Expr> {
        self.arity(args, 1, sp)?;
        let (elem, n) = self.filled_ty(targs, want, sp)?;
        if !elem.is_copy(&|_| false) {
            return Err(Diag::error(args[0].span, "type.arr_fill").kv("ty", self.show(&elem)));
        }
        let x = self.check_expr(&args[0], Some(&elem))?;
        self.expect(&x, &elem)?;
        self.filled_loop(x, elem, n, sp)
    }

    fn filled_ty(&mut self, targs: &[Ty], want: Option<&Ty>, sp: Span) -> PResult<(Ty, u64)> {
        if let Some(t) = targs.first() {
            if let Some(pair) = self.as_arr(t) {
                return Ok(pair);
            }
            let want = arr_ty(self.fresh(false), 1);
            return Err(self.mismatch(sp, &want, t));
        }
        if let Some(w) = want.and_then(|w| self.as_arr(w)) {
            return Ok(w);
        }
        Err(Diag::error(sp, "type.infer").kv("fix", "(let NAME (arr T N) ...)"))
    }

    fn filled_loop(&mut self, x: Expr, elem: Ty, n: u64, sp: Span) -> PResult<Expr> {
        self.push_scope();
        let ty = arr_ty(elem.clone(), n);
        let xid = self.declare("_x", elem.clone(), true, sp)?;
        let aid = self.declare("_a", ty.clone(), true, sp)?;
        let kid = self.declare("_k", i64_ty(), false, sp)?;
        let xlet = mk(ExprKind::Let { local: xid, init: Some(Box::new(x)) }, Ty::Unit, sp);
        let empty = mk(ExprKind::Arr(Vec::new()), ty.clone(), sp);
        let alet = mk(ExprKind::Let { local: aid, init: Some(Box::new(empty)) }, Ty::Unit, sp);
        let idx = mk(
            ExprKind::Index { base: Box::new(loc(aid, ty.clone(), sp)), idx: Box::new(loc(kid, i64_ty(), sp)) },
            elem.clone(),
            sp,
        );
        let set = mk(ExprKind::Assign { place: Box::new(idx), value: Box::new(loc(xid, elem, sp)) }, Ty::Unit, sp);
        let body = mk(ExprKind::Block(vec![set]), Ty::Unit, sp);
        let ford = mk(
            ExprKind::For { local: kid, lo: Box::new(int_lit(0, sp)), hi: Box::new(int_lit(n as i128, sp)), step: None, body: Box::new(body) },
            Ty::Unit,
            sp,
        );
        let val = loc(aid, ty.clone(), sp);
        self.pop_scope();
        Ok(mk(ExprKind::Block(vec![xlet, alet, ford, val]), ty, sp))
    }

    fn arr_len(&mut self, args: &[ast::Expr], sp: Span) -> PResult<Expr> {
        self.arity(args, 1, sp)?;
        let a = self.check_expr(&args[0], None)?;
        let (_, n) = self.ref_arr(&a, false)?;
        Ok(mk(ExprKind::Block(vec![a, int_lit(n as i128, sp)]), i64_ty(), sp))
    }

    fn arr_get(&mut self, args: &[ast::Expr], mutable: bool, sp: Span) -> PResult<Expr> {
        self.arity(args, 2, sp)?;
        let (elem, aid, iid, stmts, a_ty, n) = self.arr_head(args, mutable, sp)?;
        let base = mk(ExprKind::Deref(Box::new(loc(aid, a_ty, sp))), arr_ty(elem.clone(), n), sp);
        let idx = mk(ExprKind::Index { base: Box::new(base), idx: Box::new(loc(iid, i64_ty(), sp)) }, elem.clone(), sp);
        let rty = Ty::Ref { mutable, lt: None, inner: Box::new(elem) };
        let r = mk(ExprKind::Ref { mutable, place: Box::new(idx) }, rty.clone(), sp);
        Ok(self.close(stmts, r, rty, sp))
    }

    fn arr_set(&mut self, args: &[ast::Expr], sp: Span) -> PResult<Expr> {
        self.arity(args, 3, sp)?;
        let a = self.check_expr(&args[0], None)?;
        let (elem, n) = self.ref_arr(&a, true)?;
        let i = self.check_expr(&args[1], Some(&i64_ty()))?;
        self.expect(&i, &i64_ty())?;
        let x = self.check_expr(&args[2], Some(&elem))?;
        self.expect(&x, &elem)?;
        let a_ty = a.ty.clone();
        let (aid, iid, mut stmts) = self.bind_arr(a, i, n, sp)?;
        let base = mk(ExprKind::Deref(Box::new(loc(aid, a_ty, sp))), arr_ty(elem.clone(), n), sp);
        let idx = mk(ExprKind::Index { base: Box::new(base), idx: Box::new(loc(iid, i64_ty(), sp)) }, elem, sp);
        stmts.push(mk(ExprKind::Assign { place: Box::new(idx), value: Box::new(x) }, Ty::Unit, sp));
        self.pop_scope();
        Ok(mk(ExprKind::Block(stmts), Ty::Unit, sp))
    }

    /// Checks `(& (arr T N))` or `(&mut (arr T N))`, then binds `_a`, `_i`, and the bounds check.
    fn arr_head(&mut self, args: &[ast::Expr], need_mut: bool, sp: Span) -> PResult<(Ty, LocalId, LocalId, Vec<Expr>, Ty, u64)> {
        let a = self.check_expr(&args[0], None)?;
        let (elem, n) = self.ref_arr(&a, need_mut)?;
        let i = self.check_expr(&args[1], Some(&i64_ty()))?;
        self.expect(&i, &i64_ty())?;
        let a_ty = a.ty.clone();
        let (aid, iid, stmts) = self.bind_arr(a, i, n, sp)?;
        Ok((elem, aid, iid, stmts, a_ty, n))
    }

    fn bind_arr(&mut self, a: Expr, i: Expr, n: u64, sp: Span) -> PResult<(LocalId, LocalId, Vec<Expr>)> {
        self.push_scope();
        let aid = self.declare("_a", a.ty.clone(), false, sp)?;
        let iid = self.declare("_i", i64_ty(), false, sp)?;
        let alet = mk(ExprKind::Let { local: aid, init: Some(Box::new(a)) }, Ty::Unit, sp);
        let ilet = mk(ExprKind::Let { local: iid, init: Some(Box::new(i)) }, Ty::Unit, sp);
        let chk = self.oob_if(iid, n, sp)?;
        Ok((aid, iid, vec![alet, ilet, chk]))
    }

    fn close(&mut self, mut stmts: Vec<Expr>, tail: Expr, ty: Ty, sp: Span) -> Expr {
        self.pop_scope();
        stmts.push(tail);
        mk(ExprKind::Block(stmts), ty, sp)
    }

    fn ref_arr(&self, a: &Expr, need_mut: bool) -> PResult<(Ty, u64)> {
        let got = self.shallow(&a.ty);
        let want = if need_mut { "(&mut (arr T N))" } else { "(& (arr T N))" };
        let Ty::Ref { mutable, inner, .. } = &got else {
            return Err(Diag::error(a.span, "type.mismatch").kv("want", want).kv("got", self.show(&got)));
        };
        let Ty::Arr(elem, n) = inner.as_ref() else {
            return Err(Diag::error(a.span, "type.mismatch").kv("want", want).kv("got", self.show(&got)));
        };
        if need_mut && !*mutable {
            return Err(Diag::error(a.span, "type.mismatch").kv("want", "(&mut (arr T N))").kv("got", self.show(&got)));
        }
        Ok(((**elem).clone(), *n))
    }

    fn oob_if(&mut self, i: LocalId, n: u64, sp: Span) -> PResult<Expr> {
        let iv = loc(i, i64_ty(), sp);
        let lt = mk(ExprKind::Bin { op: BinOp::Lt, l: Box::new(iv.clone()), r: Box::new(int_lit(0, sp)) }, Ty::Bool, sp);
        let ge = mk(ExprKind::Bin { op: BinOp::Ge, l: Box::new(iv), r: Box::new(int_lit(n as i128, sp)) }, Ty::Bool, sp);
        let or = mk(ExprKind::Or(Box::new(lt), Box::new(ge)), Ty::Bool, sp);
        let panic = self.oob_panic(i, n, sp)?;
        Ok(mk(ExprKind::If { cond: Box::new(or), then: Box::new(panic), els: None }, Ty::Unit, sp))
    }

    fn oob_panic(&mut self, i: LocalId, n: u64, sp: Span) -> PResult<Expr> {
        let oob = self.std_fn("str", "oob").ok_or_else(|| Diag::error(sp, "name.unknown").kv("name", "str.oob"))?;
        let pan = self.std_fn("sys", "panic").ok_or_else(|| Diag::error(sp, "name.unknown").kv("name", "sys.panic"))?;
        let call = mk(
            ExprKind::Call { callee: Callee::Fn(oob, Vec::new()), args: vec![int_lit(n as i128, sp), loc(i, i64_ty(), sp)] },
            self.str_ty(),
            sp,
        );
        let msg = mk(ExprKind::Ref { mutable: false, place: Box::new(call) }, self.ref_str(), sp);
        Ok(mk(ExprKind::Call { callee: Callee::Fn(pan, Vec::new()), args: vec![self.loc_lit(sp), msg] }, Ty::Never, sp))
    }
}
