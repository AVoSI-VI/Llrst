//! Item parsing: S-expressions to `ast::Item`.
pub mod expr;
pub mod ty;

use crate::ast::*;
use crate::diag::{Diag, PResult, Span};
use crate::sexpr::Sexpr;
use expr::{parse_body, parse_expr};
use ty::{parse_generics, parse_ty};

fn arity(sp: Span, form: &str, want: &str, got: usize) -> Diag {
    Diag::error(sp, "parse.arity").kv("form", form).kv("want", want).kv("got", got.to_string())
}

fn name_of(s: &Sexpr) -> PResult<String> {
    s.atom()
        .filter(|a| !a.starts_with('<'))
        .map(|a| a.to_string())
        .ok_or_else(|| Diag::error(s.span(), "parse.expect").kv("want", "name").kv("got", s.show()))
}

pub fn parse_items(forms: &[Sexpr]) -> PResult<Vec<Item>> {
    forms.iter().map(|f| parse_item(f, false)).collect()
}

fn parse_item(s: &Sexpr, is_pub: bool) -> PResult<Item> {
    let sp = s.span();
    let items = s.list().ok_or_else(|| Diag::error(sp, "parse.expect").kv("want", "(form)").kv("got", s.show()))?;
    let head = items.first().and_then(|h| h.atom()).ok_or_else(|| Diag::error(sp, "parse.bad_head"))?;
    let n = items.len() - 1;
    Ok(match head {
        "pub" => {
            if n != 1 {
                return Err(arity(sp, "pub", "1", n));
            }
            let inner = items[1].head().unwrap_or("");
            if !matches!(inner, "fn" | "extern" | "struct" | "enum" | "const" | "inline" | "trait") {
                return Err(Diag::error(items[1].span(), "parse.expect")
                    .kv("want", "fn extern struct enum const inline trait")
                    .kv("got", inner)
                    .kv("fix", "(pub (fn ...)) per method"));
            }
            return parse_item(&items[1], true);
        }
        "inline" => {
            if n != 1 {
                return Err(arity(sp, "inline", "1", n));
            }
            return Ok(Item::Fn(inline_fn(&items[1], is_pub)?));
        }
        "use" => {
            if n != 1 {
                return Err(arity(sp, "use", "1", n));
            }
            let path = name_of(&items[1])?;
            Item::Use(path.split('.').map(|s| s.to_string()).collect(), sp)
        }
        "mod" => {
            if n != 1 {
                return Err(arity(sp, "mod", "1", n));
            }
            Item::Mod(name_of(&items[1])?, sp)
        }
        "fn" => Item::Fn(parse_fn(items, sp, false, is_pub)?),
        "extern" => Item::Fn(parse_fn(items, sp, true, is_pub)?),
        "struct" => {
            let name = name_of(items.get(1).ok_or_else(|| arity(sp, "struct", "1+", n))?)?;
            let (generics, at) = parse_generics(items, 2)?;
            let mut fields = Vec::new();
            for f in &items[at..] {
                let (is_pub, inner) = if f.head() == Some("pub") {
                    let l = f.list().unwrap();
                    if l.len() != 2 {
                        return Err(arity(f.span(), "pub", "1", l.len().saturating_sub(1)));
                    }
                    (true, &l[1])
                } else {
                    (false, f)
                };
                let l = inner.list().filter(|l| l.len() == 2).ok_or_else(|| {
                    Diag::error(inner.span(), "parse.expect").kv("want", "(name type)").kv("got", inner.show())
                })?;
                fields.push(Field { name: name_of(&l[0])?, ty: parse_ty(&l[1])?, is_pub });
            }
            Item::Struct(StructDecl { name, generics, fields, is_pub, span: sp })
        }
        "enum" => {
            let name = name_of(items.get(1).ok_or_else(|| arity(sp, "enum", "2+", n))?)?;
            let (generics, at) = parse_generics(items, 2)?;
            let mut variants = Vec::new();
            for v in &items[at..] {
                let (is_pub, inner) = if v.head() == Some("pub") {
                    let l = v.list().unwrap();
                    if l.len() != 2 {
                        return Err(arity(v.span(), "pub", "1", l.len().saturating_sub(1)));
                    }
                    (true, &l[1])
                } else {
                    (false, v)
                };
                let l = inner.list().filter(|l| !l.is_empty()).ok_or_else(|| {
                    Diag::error(inner.span(), "parse.expect").kv("want", "(name type*)").kv("got", inner.show())
                })?;
                let fields = l[1..].iter().map(parse_ty).collect::<PResult<Vec<_>>>()?;
                variants.push(Variant { name: name_of(&l[0])?, fields, is_pub, span: inner.span() });
            }
            if variants.is_empty() {
                return Err(arity(sp, "enum", "2+", n));
            }
            Item::Enum(EnumDecl { name, generics, variants, is_pub, span: sp })
        }
        "impl" => {
            if is_trait_impl(items) {
                let trait_name = name_of(items.get(1).ok_or_else(|| arity(sp, "impl", "2+", n))?)?;
                let ty_name = name_of(&items[2])?;
                let mut methods = Vec::new();
                for f in &items[3..] {
                    methods.push(method_fn(f)?);
                }
                Item::TraitImpl(TraitImpl { trait_name, ty_name, methods, span: sp })
            } else {
                let ty_name = name_of(items.get(1).ok_or_else(|| arity(sp, "impl", "1+", n))?)?;
                let (generics, at) = parse_generics(items, 2)?;
                let mut fns = Vec::new();
                for f in &items[at..] {
                    fns.push(method_fn(f)?);
                }
                Item::Impl(ImplDecl { ty_name, generics, fns, span: sp })
            }
        }
        "trait" => {
            let name = name_of(items.get(1).ok_or_else(|| arity(sp, "trait", "1+", n))?)?;
            let mut methods = Vec::new();
            for f in &items[2..] {
                methods.push(trait_method(f)?);
            }
            Item::Trait(TraitDecl { name, methods, is_pub, span: sp })
        }
        "const" => {
            if n != 3 {
                return Err(arity(sp, "const", "3", n));
            }
            Item::Const(ConstDecl { name: name_of(&items[1])?, ty: parse_ty(&items[2])?, value: parse_expr(&items[3])?, is_pub, span: sp })
        }
        "test" => {
            let name = name_of(items.get(1).ok_or_else(|| arity(sp, "test", "1+", n))?)?;
            Item::Test(TestDecl { name, body: parse_body(&items[2..])?, span: sp })
        }
        _ => return Err(Diag::error(items[0].span(), "parse.bad_head").kv("got", head)),
    })
}

fn parse_fn(items: &[Sexpr], sp: Span, is_extern: bool, is_pub: bool) -> PResult<FnDecl> {
    let n = items.len() - 1;
    let form = if is_extern { "extern" } else { "fn" };
    let name = name_of(items.get(1).ok_or_else(|| arity(sp, form, "3+", n))?)?;
    let (generics, at) = parse_generics(items, 2)?;
    let plist = items.get(at).and_then(|p| p.list()).ok_or_else(|| {
        Diag::error(items.get(at).map_or(sp, |x| x.span()), "parse.expect").kv("want", "((name type)...)")
    })?;
    let mut params = Vec::new();
    for p in plist {
        let l = p.list().filter(|l| l.len() == 2).ok_or_else(|| {
            Diag::error(p.span(), "parse.expect").kv("want", "(name type)").kv("got", p.show())
        })?;
        params.push(Param { name: name_of(&l[0])?, ty: parse_ty(&l[1])?, span: p.span() });
    }
    let ret = parse_ty(items.get(at + 1).ok_or_else(|| arity(sp, form, "3+", n))?)?;
    let body = parse_body(&items[at + 2..])?;
    if is_extern && !body.is_empty() {
        return Err(arity(sp, "extern", "3", n));
    }
    Ok(FnDecl { name, generics, params, ret, body, is_extern, is_pub, inline: false, span: sp })
}

/// `(inline (fn ...))` or `(inline (extern ...))`.
fn inline_fn(inner: &Sexpr, is_pub: bool) -> PResult<FnDecl> {
    let head = inner.head().unwrap_or("");
    if !matches!(head, "fn" | "extern") {
        return Err(Diag::error(inner.span(), "parse.expect").kv("want", "(fn ...)").kv("got", inner.show()));
    }
    let list = inner.list().ok_or_else(|| Diag::error(inner.span(), "parse.expect").kv("want", "(fn ...)"))?;
    let mut f = parse_fn(list, inner.span(), head == "extern", is_pub)?;
    f.inline = true;
    Ok(f)
}

/// `(fn eq ((& Self) (& Self)) bool)` : parameters are types, not bindings.
fn trait_method(s: &Sexpr) -> PResult<FnDecl> {
    let l = s.list().ok_or_else(|| Diag::error(s.span(), "parse.expect").kv("want", "(fn ...)"))?;
    if !l.first().map_or(false, |h| h.is_atom("fn")) || l.len() < 4 {
        return Err(Diag::error(s.span(), "parse.expect").kv("want", "(fn name (types) ret)").kv("got", s.show()));
    }
    let plist = l[2].list().ok_or_else(|| Diag::error(l[2].span(), "parse.expect").kv("want", "(types)"))?;
    let mut params = Vec::new();
    for (i, p) in plist.iter().enumerate() {
        params.push(Param { name: format!("_{i}"), ty: parse_ty(p)?, span: p.span() });
    }
    Ok(FnDecl {
        name: name_of(&l[1])?,
        generics: Generics { types: Vec::new(), bounds: Vec::new(), lifetimes: Vec::new() },
        params,
        ret: parse_ty(&l[3])?,
        body: Vec::new(),
        is_extern: false,
        is_pub: true,
        inline: false,
        span: s.span(),
    })
}

fn is_trait_impl(items: &[Sexpr]) -> bool {
    match items.get(2).and_then(|s| s.atom()) {
        Some(a) => !a.starts_with('<'),
        None => false,
    }
}

/// A method is `(fn ...)`, `(pub (fn ...))`, or `(pub (inline (fn ...)))`.
fn method_fn(s: &Sexpr) -> PResult<FnDecl> {
    let mut cur = s;
    let mut is_pub = false;
    if cur.head() == Some("pub") {
        cur = peel(cur, "pub")?;
        is_pub = true;
    }
    let mut inline = false;
    if cur.head() == Some("inline") {
        cur = peel(cur, "inline")?;
        inline = true;
    }
    let l = cur.list().ok_or_else(|| Diag::error(cur.span(), "parse.expect").kv("want", "(fn ...)"))?;
    if !l.first().map_or(false, |h| h.is_atom("fn")) {
        return Err(Diag::error(cur.span(), "parse.expect").kv("want", "(fn ...)").kv("got", cur.show()));
    }
    let mut f = parse_fn(l, s.span(), false, is_pub)?;
    f.inline = inline;
    Ok(f)
}

fn peel<'a>(s: &'a Sexpr, form: &str) -> PResult<&'a Sexpr> {
    let l = s.list().ok_or_else(|| Diag::error(s.span(), "parse.expect").kv("want", form))?;
    if l.len() != 2 {
        return Err(arity(s.span(), form, "1", l.len().saturating_sub(1)));
    }
    Ok(&l[1])
}
