//! Type expression and generics parsing.
use crate::ast::{Generics, TyExpr};
use crate::diag::{Diag, PResult};
use crate::sexpr::Sexpr;

pub fn parse_ty(s: &Sexpr) -> PResult<TyExpr> {
    match s {
        Sexpr::Atom(a, sp) => {
            if a.starts_with('\'') || a.starts_with('<') {
                return Err(Diag::error(*sp, "parse.expect").kv("want", "type").kv("got", a.clone()));
            }
            Ok(TyExpr::Name(a.clone(), *sp))
        }
        Sexpr::Str(_, sp) => Err(Diag::error(*sp, "parse.expect").kv("want", "type").kv("got", "string")),
        Sexpr::List(items, sp) => {
            let head = items.first().and_then(|h| h.atom()).ok_or_else(|| {
                Diag::error(*sp, "parse.expect").kv("want", "type").kv("got", s.show())
            })?;
            match head {
                "&" | "&mut" => {
                    let mutable = head == "&mut";
                    let (lt, rest) = match items.get(1).and_then(|x| x.atom()) {
                        Some(a) if a.starts_with('\'') => (Some(a.to_string()), &items[2..]),
                        _ => (None, &items[1..]),
                    };
                    if rest.len() != 1 {
                        return Err(Diag::error(*sp, "parse.arity").kv("form", head).kv("want", "1").kv("got", rest.len().to_string()));
                    }
                    Ok(TyExpr::Ref { mutable, lt, inner: Box::new(parse_ty(&rest[0])?), span: *sp })
                }
                "ptr" => {
                    if items.len() != 2 {
                        return Err(Diag::error(*sp, "parse.arity").kv("form", "ptr").kv("want", "1").kv("got", (items.len() - 1).to_string()));
                    }
                    Ok(TyExpr::Ptr(Box::new(parse_ty(&items[1])?), *sp))
                }
                "fn" => {
                    if items.len() != 3 {
                        return Err(Diag::error(*sp, "parse.arity").kv("form", "fn type").kv("want", "2").kv("got", (items.len() - 1).to_string()));
                    }
                    let params = items[1].list().ok_or_else(|| Diag::error(items[1].span(), "parse.expect").kv("want", "(types)"))?;
                    let ps = params.iter().map(parse_ty).collect::<PResult<Vec<_>>>()?;
                    Ok(TyExpr::Fn(ps, Box::new(parse_ty(&items[2])?), *sp))
                }
                _ => {
                    let args = items[1..].iter().map(parse_ty).collect::<PResult<Vec<_>>>()?;
                    Ok(TyExpr::App(head.to_string(), args, *sp))
                }
            }
        }
    }
}

/// Parses `<T U 'a>` written as consecutive atoms. Returns the generics and the index
/// of the first form after them. Accepts the absence of generics.
pub fn parse_generics(items: &[Sexpr], at: usize) -> PResult<(Generics, usize)> {
    let mut g = Generics { types: Vec::new(), lifetimes: Vec::new() };
    let Some(first) = items.get(at).and_then(|x| x.atom()) else { return Ok((g, at)) };
    if !first.starts_with('<') {
        return Ok((g, at));
    }
    let mut i = at;
    loop {
        let Some(a) = items.get(i).and_then(|x| x.atom()) else {
            return Err(Diag::error(items[at].span(), "parse.expect").kv("want", ">"));
        };
        let mut name = a.trim_start_matches('<');
        let last = name.ends_with('>');
        name = name.trim_end_matches('>');
        if !name.is_empty() {
            if let Some(lt) = name.strip_prefix('\'') {
                g.lifetimes.push(lt.to_string());
            } else {
                g.types.push(name.to_string());
            }
        }
        i += 1;
        if last {
            return Ok((g, i));
        }
    }
}

/// Explicit type arguments at a call site: `(f <i64 str> ...)`. Returns parsed types and
/// index after them.
pub fn parse_targs(items: &[Sexpr], at: usize) -> PResult<(Vec<TyExpr>, usize)> {
    let Some(first) = items.get(at).and_then(|x| x.atom()) else { return Ok((Vec::new(), at)) };
    if !first.starts_with('<') {
        return Ok((Vec::new(), at));
    }
    // Rebuild the atoms between < and > into a flat token list, then reparse via a
    // synthetic sexpr. Type args may be lists like (vec str); those appear as List items.
    let mut out = Vec::new();
    let mut i = at;
    let mut pending: Vec<Sexpr> = Vec::new();
    loop {
        let Some(item) = items.get(i) else {
            return Err(Diag::error(items[at].span(), "parse.expect").kv("want", ">"));
        };
        i += 1;
        match item {
            Sexpr::Atom(a, sp) => {
                let mut name = a.as_str();
                if i - 1 == at {
                    name = name.trim_start_matches('<');
                }
                let last = name.ends_with('>');
                name = name.trim_end_matches('>');
                if !name.is_empty() {
                    pending.push(Sexpr::Atom(name.to_string(), *sp));
                }
                if last {
                    break;
                }
            }
            other => pending.push(other.clone()),
        }
    }
    for p in &pending {
        out.push(parse_ty(p)?);
    }
    Ok((out, i))
}
