//! Expression and pattern parsing.
use super::ty::{parse_targs, parse_ty};
use crate::ast::{Arm, BinOp, Expr, ExprKind, Lit, Param, Pat};
use crate::diag::{Diag, PResult, Span};
use crate::sexpr::Sexpr;

fn arity(sp: Span, form: &str, want: &str, got: usize) -> Diag {
    Diag::error(sp, "parse.arity").kv("form", form).kv("want", want).kv("got", got.to_string())
}

fn mk(kind: ExprKind, span: Span) -> Expr {
    Expr { kind, span }
}

fn bx(e: Expr) -> Box<Expr> {
    Box::new(e)
}

fn is_type_name(head: &str) -> bool {
    let mut chars = head.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !matches!(
            head,
            "let" | "set" | "do" | "if" | "while" | "loop" | "for" | "brk" | "cont" | "ret" | "match" | "as" | "and"
                | "or" | "not" | "neg" | "wneg" | "bnot" | "try" | "panic" | "assert" | "assert_eq" | "assert_ne" | "assert_near" | "drop" | "sizeof" | "call"
        )
}

fn ref_tail(items: &[Sexpr]) -> &[Sexpr] {
    let rest = &items[1..];
    match rest.first() {
        Some(Sexpr::Atom(a, _)) if a.starts_with('\'') => &rest[1..],
        _ => rest,
    }
}

fn is_type_form(s: &Sexpr) -> bool {
    match s {
        Sexpr::Atom(a, _) => matches!(
            a.as_str(),
            "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32" | "f64" | "bool" | "str" | "unit"
        ),
        Sexpr::List(items, _) => {
            let Some(head) = items.first().and_then(|h| h.atom()) else { return false };
            if matches!(head, "ptr" | "fn" | "clos") {
                return true;
            }
            if matches!(head, "&" | "&mut") {
                let rest = ref_tail(items);
                return !rest.is_empty() && rest.iter().all(is_type_form);
            }
            is_type_name(head) && items.len() > 1 && items[1..].iter().all(is_type_form)
        }
        Sexpr::Str(_, _) => false,
    }
}

pub fn parse_body(items: &[Sexpr]) -> PResult<Vec<Expr>> {
    items.iter().map(parse_expr).collect()
}

pub fn parse_lit_atom(a: &str) -> Option<Lit> {
    match a {
        "true" => return Some(Lit::Bool(true)),
        "false" => return Some(Lit::Bool(false)),
        "()" => return Some(Lit::Unit),
        _ => {}
    }
    let c = a.as_bytes()[0];
    if !(c.is_ascii_digit() || (c == b'-' && a.len() > 1 && a.as_bytes()[1].is_ascii_digit())) {
        return None;
    }
    let body_end = a
        .char_indices()
        .find(|&(i, ch)| i > 0 && (ch == 'i' || ch == 'u' || ch == 'f'))
        .map(|(i, _)| i)
        .unwrap_or(a.len());
    let (raw, suffix) = a.split_at(body_end);
    let suffix = if suffix.is_empty() { None } else { Some(suffix.to_string()) };
    let flat: String = raw.chars().filter(|c| *c != '_').collect();
    let neg = flat.starts_with('-');
    let body = if neg { &flat[1..] } else { flat.as_str() };
    if body.is_empty() {
        return None;
    }
    let radix = if body.starts_with("0x") {
        Some(16)
    } else if body.starts_with("0b") {
        Some(2)
    } else if body.starts_with("0o") {
        Some(8)
    } else {
        None
    };
    let float = radix.is_none()
        && (body.contains('.') || body.contains('e') || body.contains('E') || suffix.as_deref().map_or(false, |s| s.starts_with('f')));
    if float {
        return body.parse::<f64>().ok().map(|v| Lit::Float(if neg { -v } else { v }, suffix));
    }
    int_lit(body, radix, neg, suffix)
}

/// `i128` when the magnitude fits, including a negative sign. Otherwise a non-negative `Lit::Wide`.
fn int_lit(body: &str, radix: Option<u32>, neg: bool, suffix: Option<String>) -> Option<Lit> {
    let digits = match radix {
        Some(_) => &body[2..],
        None => body,
    };
    let base = radix.unwrap_or(10);
    if let Ok(v) = i128::from_str_radix(digits, base) {
        let v = if neg { v.checked_neg()? } else { v };
        return Some(Lit::Int(v, suffix));
    }
    if neg {
        return None;
    }
    u128::from_str_radix(digits, base).ok().map(|v| Lit::Wide(v, suffix))
}

pub fn parse_expr(s: &Sexpr) -> PResult<Expr> {
    let sp = s.span();
    match s {
        Sexpr::Str(b, _) => Ok(mk(ExprKind::Lit(Lit::Str(b.clone())), sp)),
        Sexpr::Atom(a, _) => {
            if let Some(l) = parse_lit_atom(a) {
                return Ok(mk(ExprKind::Lit(l), sp));
            }
            Ok(mk(ExprKind::Name(a.clone()), sp))
        }
        Sexpr::List(items, _) => {
            if items.is_empty() {
                return Ok(mk(ExprKind::Lit(Lit::Unit), sp));
            }
            let Some(head) = items[0].atom() else {
                return Err(Diag::error(sp, "parse.bad_head").kv("got", items[0].show()));
            };
            parse_form(head, items, sp)
        }
    }
}

fn one(items: &[Sexpr], sp: Span, form: &str) -> PResult<Box<Expr>> {
    if items.len() != 2 {
        return Err(arity(sp, form, "1", items.len() - 1));
    }
    Ok(bx(parse_expr(&items[1])?))
}

fn two(items: &[Sexpr], sp: Span, form: &str) -> PResult<(Box<Expr>, Box<Expr>)> {
    if items.len() != 3 {
        return Err(arity(sp, form, "2", items.len() - 1));
    }
    Ok((bx(parse_expr(&items[1])?), bx(parse_expr(&items[2])?)))
}

fn parse_form(head: &str, items: &[Sexpr], sp: Span) -> PResult<Expr> {
    let n = items.len() - 1;
    let k = match head {
        "let" => {
            let mutable = items.get(1).map_or(false, |x| x.is_atom("mut"));
            let rest = &items[1 + mutable as usize..];
            let name = rest.first().and_then(|x| x.atom()).ok_or_else(|| arity(sp, "let", "2..3", n))?;
            if rest.len() == 2 && is_type_form(&rest[1]) {
                if !mutable {
                    return Err(Diag::error(sp, "mut.required").kv("name", name).kv("fix", format!("(let mut {name} ...)")));
                }
                ExprKind::Let { name: name.to_string(), mutable, ty: Some(parse_ty(&rest[1])?), init: None }
            } else {
                let (ty, init) = match rest.len() {
                    2 => (None, &rest[1]),
                    3 => (Some(parse_ty(&rest[1])?), &rest[2]),
                    _ => return Err(arity(sp, "let", "2..3", n)),
                };
                ExprKind::Let { name: name.to_string(), mutable, ty, init: Some(bx(parse_expr(init)?)) }
            }
        }
        "set" if n == 1 => {
            let (targs, at) = parse_targs(items, 1)?;
            ExprKind::Call { callee: head.to_string(), targs, args: parse_body(&items[at..])? }
        }
        "set" => {
            let (p, v) = two(items, sp, "set")?;
            ExprKind::Set { place: p, value: v }
        }
        "do" => ExprKind::Do(parse_body(&items[1..])?),
        "if" => {
            if n != 2 && n != 3 {
                return Err(arity(sp, "if", "2..3", n));
            }
            let els = if n == 3 { Some(bx(parse_expr(&items[3])?)) } else { None };
            ExprKind::If { cond: bx(parse_expr(&items[1])?), then: bx(parse_expr(&items[2])?), els }
        }
        "while" => {
            if n < 1 {
                return Err(arity(sp, "while", "1+", n));
            }
            ExprKind::While { cond: bx(parse_expr(&items[1])?), body: parse_body(&items[2..])? }
        }
        "loop" => {
            let (label, rest) = loop_head(items);
            ExprKind::Loop { label, body: parse_body(rest)? }
        }
        "for" => parse_for(items, sp, n)?,
        "brk" | "cont" => parse_jump(head, items, sp, n)?,
        "ret" => match n {
            0 => ExprKind::Return(None),
            1 => ExprKind::Return(Some(bx(parse_expr(&items[1])?))),
            _ => return Err(arity(sp, "ret", "0..1", n)),
        },
        "match" => {
            if n < 2 {
                return Err(arity(sp, "match", "2+", n));
            }
            let scrut = bx(parse_expr(&items[1])?);
            let mut arms = Vec::new();
            for a in &items[2..] {
                arms.push(parse_arm(a)?);
            }
            ExprKind::Match { scrut, arms }
        }
        "&" | "&mut" => ExprKind::Ref { mutable: head == "&mut", place: one(items, sp, head)? },
        "@" => ExprKind::Deref(one(items, sp, "@")?),
        "." => {
            if n != 2 {
                return Err(arity(sp, ".", "2", n));
            }
            let name = items[2].atom().ok_or_else(|| Diag::error(items[2].span(), "parse.expect").kv("want", "field"))?;
            ExprKind::Field { base: bx(parse_expr(&items[1])?), name: name.to_string() }
        }
        "as" => {
            if n != 2 {
                return Err(arity(sp, "as", "2", n));
            }
            ExprKind::Cast { ty: parse_ty(&items[1])?, expr: bx(parse_expr(&items[2])?) }
        }
        "and" | "or" => {
            if n < 2 {
                return Err(arity(sp, head, "2+", n));
            }
            let mut acc = parse_expr(&items[1])?;
            for it in &items[2..] {
                let r = parse_expr(it)?;
                let k = if head == "and" { ExprKind::And(bx(acc), bx(r)) } else { ExprKind::Or(bx(acc), bx(r)) };
                acc = mk(k, sp);
            }
            return Ok(acc);
        }
        "not" => ExprKind::Not(one(items, sp, "not")?),
        "neg" => ExprKind::Neg(one(items, sp, "neg")?),
        "wneg" => ExprKind::WNeg(one(items, sp, "wneg")?),
        "bnot" => ExprKind::BNot(one(items, sp, "bnot")?),
        "try" => ExprKind::Try(one(items, sp, "try")?),
        "panic" => ExprKind::Panic(one(items, sp, "panic")?),
        "assert" => ExprKind::Assert(one(items, sp, "assert")?),
        "assert_eq" => {
            let (l, r) = two(items, sp, "assert_eq")?;
            ExprKind::AssertEq(l, r)
        }
        "assert_ne" => {
            let (l, r) = two(items, sp, "assert_ne")?;
            ExprKind::AssertNe(l, r)
        }
        "assert_near" => {
            if items.len() != 4 {
                return Err(arity(sp, "assert_near", "3", items.len() - 1));
            }
            ExprKind::AssertNear(bx(parse_expr(&items[1])?), bx(parse_expr(&items[2])?), bx(parse_expr(&items[3])?))
        }
        "drop" => ExprKind::Drop(one(items, sp, "drop")?),
        "sizeof" => {
            if n != 1 {
                return Err(arity(sp, "sizeof", "1", n));
            }
            ExprKind::Sizeof(parse_ty(&items[1])?)
        }
        "call" => {
            if n < 1 {
                return Err(arity(sp, "call", "1+", n));
            }
            ExprKind::CallPtr { callee: bx(parse_expr(&items[1])?), args: parse_body(&items[2..])? }
        }
        "clos" => parse_clos(items, sp)?,
        _ => {
            if let Some(op) = BinOp::from_str(head) {
                let (l, r) = two(items, sp, head)?;
                ExprKind::Bin { op, l, r }
            } else {
                let (targs, at) = parse_targs(items, 1)?;
                ExprKind::Call { callee: head.to_string(), targs, args: parse_body(&items[at..])? }
            }
        }
    };
    Ok(mk(k, sp))
}

/// A leading name is the label when another form follows it. A lone name is the body.
fn loop_head(items: &[Sexpr]) -> (Option<String>, &[Sexpr]) {
    if items.len() >= 3 {
        if let Sexpr::Atom(a, _) = &items[1] {
            if parse_lit_atom(a).is_none() {
                return (Some(a.clone()), &items[2..]);
            }
        }
    }
    (None, &items[1..])
}

fn parse_for(items: &[Sexpr], sp: Span, n: usize) -> PResult<ExprKind> {
    if n < 3 {
        return Err(arity(sp, "for", "3+", n));
    }
    let name = items[1].atom().ok_or_else(|| Diag::error(items[1].span(), "parse.expect").kv("want", "name"))?;
    let mut at = 4;
    let mut step = None;
    if items.len() > 5 && matches!(items.get(4), Some(Sexpr::Atom(_, _))) {
        step = Some(bx(parse_expr(&items[4])?));
        at = 5;
    }
    Ok(ExprKind::For {
        name: name.to_string(),
        lo: bx(parse_expr(&items[2])?),
        hi: bx(parse_expr(&items[3])?),
        step,
        body: parse_body(&items[at..])?,
    })
}

fn parse_jump(head: &str, items: &[Sexpr], sp: Span, n: usize) -> PResult<ExprKind> {
    if n > 1 {
        return Err(arity(sp, head, "0..1", n));
    }
    let label = if n == 1 {
        let a = items[1].atom().filter(|a| parse_lit_atom(a).is_none()).ok_or_else(|| {
            Diag::error(items[1].span(), "parse.expect").kv("want", "name").kv("got", items[1].show())
        })?;
        Some(a.to_string())
    } else {
        None
    };
    Ok(if head == "brk" { ExprKind::Break(label) } else { ExprKind::Continue(label) })
}

fn parse_clos(items: &[Sexpr], sp: Span) -> PResult<ExprKind> {
    let n = items.len() - 1;
    if n < 2 {
        return Err(arity(sp, "clos", "2+", n));
    }
    let plist = items[1].list().ok_or_else(|| Diag::error(items[1].span(), "parse.expect").kv("want", "((name type)...)"))?;
    let mut params = Vec::new();
    for p in plist {
        let l = p.list().filter(|l| l.len() == 2).ok_or_else(|| Diag::error(p.span(), "parse.expect").kv("want", "(name type)"))?;
        let name = l[0].atom().ok_or_else(|| Diag::error(l[0].span(), "parse.expect").kv("want", "name"))?.to_string();
        params.push(Param { name, ty: parse_ty(&l[1])?, span: p.span() });
    }
    let ret = parse_ty(&items[2])?;
    Ok(ExprKind::Clos { params, ret, body: parse_body(&items[3..])? })
}

fn parse_arm(a: &Sexpr) -> PResult<Arm> {
    let l = a.list().ok_or_else(|| Diag::error(a.span(), "parse.expect").kv("want", "(pat expr)").kv("got", a.show()))?;
    if l.len() == 2 {
        return Ok(Arm { pat: parse_pat(&l[0])?, guard: None, body: parse_expr(&l[1])? });
    }
    if l.len() == 3 {
        let g = l[1].list().filter(|g| g.len() == 2 && g[0].atom() == Some("if")).ok_or_else(|| {
            Diag::error(l[1].span(), "parse.expect").kv("want", "(if expr)").kv("got", l[1].show())
        })?;
        return Ok(Arm { pat: parse_pat(&l[0])?, guard: Some(parse_expr(&g[1])?), body: parse_expr(&l[2])? });
    }
    Err(Diag::error(a.span(), "parse.expect").kv("want", "(pat expr)").kv("got", a.show()))
}

pub fn parse_pat(s: &Sexpr) -> PResult<Pat> {
    let sp = s.span();
    match s {
        Sexpr::Str(b, _) => Ok(Pat::Lit(Lit::Str(b.clone()), sp)),
        Sexpr::Atom(a, _) => {
            if a == "_" {
                return Ok(Pat::Wild(sp));
            }
            if let Some(l) = parse_lit_atom(a) {
                return Ok(Pat::Lit(l, sp));
            }
            if a.contains('.') {
                return Ok(Pat::Variant { path: a.clone(), subs: Vec::new(), span: sp });
            }
            Ok(Pat::Bind(a.clone(), sp))
        }
        Sexpr::List(items, _) => {
            let head = items.first().and_then(|h| h.atom()).ok_or_else(|| {
                Diag::error(sp, "parse.expect").kv("want", "pattern")
            })?;
            if head == "or" {
                if items.len() < 3 {
                    return Err(arity(sp, "or", "2+", items.len() - 1));
                }
                let subs = items[1..].iter().map(parse_pat).collect::<PResult<Vec<_>>>()?;
                return Ok(Pat::Or { subs, span: sp });
            }
            let subs = items[1..].iter().map(parse_pat).collect::<PResult<Vec<_>>>()?;
            if head.contains('.') {
                Ok(Pat::Variant { path: head.to_string(), subs, span: sp })
            } else {
                Ok(Pat::Struct { path: head.to_string(), subs, span: sp })
            }
        }
    }
}
