//! `(arr T N)` resolution and the `extern` rejection.
use crate::ast::TyExpr;
use crate::diag::{Diag, PResult, Span};
use crate::resolve::{resolve_ty, Program};
use crate::ty::{DefId, Ty, TyPrinter};

pub fn resolve_arr(prog: &Program, module: u32, generics: &[String], args: &[TyExpr], sp: Span) -> PResult<Ty> {
    if args.len() != 2 {
        return Err(Diag::error(sp, "type.generic_arity").kv("name", "arr").kv("want", "2").kv("got", args.len().to_string()));
    }
    let elem = resolve_ty(prog, module, generics, &args[0])?;
    let n = arr_len(&args[1])?;
    Ok(Ty::Arr(Box::new(elem), n))
}

/// `N` in `(arr T N)`: a decimal integer atom ≥ 1 that fits in `i64`, no suffix, no `_`.
fn arr_len(t: &TyExpr) -> PResult<u64> {
    let TyExpr::Name(n, sp) = t else {
        return Err(Diag::error(t.span(), "type.arr_len").kv("got", ty_got(t)));
    };
    let ok = !n.is_empty() && !n.starts_with('0') && n.bytes().all(|b| b.is_ascii_digit());
    if ok {
        if let Ok(v) = n.parse::<u64>() {
            if v <= i64::MAX as u64 {
                return Ok(v);
            }
        }
    }
    Err(Diag::error(*sp, "type.arr_len").kv("got", n.clone()))
}

fn ty_got(t: &TyExpr) -> String {
    match t {
        TyExpr::Name(n, _) => n.clone(),
        TyExpr::App(n, args, _) => {
            let a: Vec<String> = args.iter().map(ty_got).collect();
            format!("({} {})", n, a.join(" "))
        }
        TyExpr::Ref { mutable, inner, .. } => format!("({} {})", if *mutable { "&mut" } else { "&" }, ty_got(inner)),
        TyExpr::Ptr(inner, _) => format!("(ptr {})", ty_got(inner)),
        _ => "ty".into(),
    }
}

fn find_arr(t: &Ty) -> Option<&Ty> {
    match t {
        Ty::Arr(..) => Some(t),
        Ty::Ref { inner, .. } | Ty::Ptr(inner) => find_arr(inner),
        Ty::Adt(_, a) => a.iter().find_map(find_arr),
        Ty::Fn(ps, r) | Ty::Clos(ps, r) => ps.iter().find_map(find_arr).or_else(|| find_arr(r)),
        _ => None,
    }
}

pub fn extern_arr(prog: &Program, t: &Ty, sp: Span) -> PResult<()> {
    let Some(a) = find_arr(t) else { return Ok(()) };
    let shown = TyPrinter { names: &|d: DefId| prog.type_name(d) }.show(a);
    Err(Diag::error(sp, "type.extern_ty").kv("ty", shown))
}
