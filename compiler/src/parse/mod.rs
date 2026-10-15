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
            return parse_item(&items[1], true);
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
                let l = f.list().filter(|l| l.len() == 2).ok_or_else(|| {
                    Diag::error(f.span(), "parse.expect").kv("want", "(name type)").kv("got", f.show())
                })?;
                fields.push(Field { name: name_of(&l[0])?, ty: parse_ty(&l[1])? });
            }
            Item::Struct(StructDecl { name, generics, fields, span: sp })
        }
        "enum" => {
            let name = name_of(items.get(1).ok_or_else(|| arity(sp, "enum", "2+", n))?)?;
            let (generics, at) = parse_generics(items, 2)?;
            let mut variants = Vec::new();
            for v in &items[at..] {
                let l = v.list().filter(|l| !l.is_empty()).ok_or_else(|| {
                    Diag::error(v.span(), "parse.expect").kv("want", "(name type*)").kv("got", v.show())
                })?;
                let fields = l[1..].iter().map(parse_ty).collect::<PResult<Vec<_>>>()?;
                variants.push(Variant { name: name_of(&l[0])?, fields, span: v.span() });
            }
            if variants.is_empty() {
                return Err(arity(sp, "enum", "2+", n));
            }
            Item::Enum(EnumDecl { name, generics, variants, span: sp })
        }
        "impl" => {
            let ty_name = name_of(items.get(1).ok_or_else(|| arity(sp, "impl", "1+", n))?)?;
            let (generics, at) = parse_generics(items, 2)?;
            let mut fns = Vec::new();
            for f in &items[at..] {
                let l = f.list().ok_or_else(|| Diag::error(f.span(), "parse.expect").kv("want", "(fn ...)"))?;
                let fpub = l.first().map_or(false, |h| h.is_atom("pub"));
                let l = if fpub { l[1].list().unwrap_or(&[]) } else { l };
                if !l.first().map_or(false, |h| h.is_atom("fn")) {
                    return Err(Diag::error(f.span(), "parse.expect").kv("want", "(fn ...)").kv("got", f.show()));
                }
                fns.push(parse_fn(l, f.span(), false, fpub)?);
            }
            Item::Impl(ImplDecl { ty_name, generics, fns, span: sp })
        }
        "const" => {
            if n != 3 {
                return Err(arity(sp, "const", "3", n));
            }
            Item::Const(ConstDecl { name: name_of(&items[1])?, ty: parse_ty(&items[2])?, value: parse_expr(&items[3])?, span: sp })
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
    Ok(FnDecl { name, generics, params, ret, body, is_extern, is_pub, span: sp })
}
