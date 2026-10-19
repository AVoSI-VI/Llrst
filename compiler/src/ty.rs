//! Semantic types.
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntTy {
    I8, I16, I32, I64, U8, U16, U32, U64,
}

impl IntTy {
    pub fn from_name(s: &str) -> Option<IntTy> {
        Some(match s {
            "i8" => IntTy::I8, "i16" => IntTy::I16, "i32" => IntTy::I32, "i64" => IntTy::I64,
            "u8" => IntTy::U8, "u16" => IntTy::U16, "u32" => IntTy::U32, "u64" => IntTy::U64,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            IntTy::I8 => "i8", IntTy::I16 => "i16", IntTy::I32 => "i32", IntTy::I64 => "i64",
            IntTy::U8 => "u8", IntTy::U16 => "u16", IntTy::U32 => "u32", IntTy::U64 => "u64",
        }
    }
    pub fn bytes(self) -> u32 {
        match self {
            IntTy::I8 | IntTy::U8 => 1,
            IntTy::I16 | IntTy::U16 => 2,
            IntTy::I32 | IntTy::U32 => 4,
            IntTy::I64 | IntTy::U64 => 8,
        }
    }
    pub fn signed(self) -> bool {
        matches!(self, IntTy::I8 | IntTy::I16 | IntTy::I32 | IntTy::I64)
    }
    pub fn fits(self, v: i128) -> bool {
        let bits = self.bytes() as u32 * 8;
        if self.signed() {
            let min = -(1i128 << (bits - 1));
            let max = (1i128 << (bits - 1)) - 1;
            v >= min && v <= max
        } else {
            v >= 0 && v < (1i128 << bits)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ty {
    Int(IntTy),
    F32,
    F64,
    Bool,
    Unit,
    Never,
    Ref { mutable: bool, lt: Option<String>, inner: Box<Ty> },
    Ptr(Box<Ty>),
    /// Struct or enum instance.
    Adt(DefId, Vec<Ty>),
    Fn(Vec<Ty>, Box<Ty>),
    /// Owned closure. The environment is erased; the value is call code, drop code, and a heap env.
    Clos(Vec<Ty>, Box<Ty>),
    /// Fat pointer: data, then vtable. `mutable` is the reference's mutability.
    Dyn { mutable: bool, trait_id: DefId },
    /// The trait's implementing type, only inside a trait signature.
    SelfTy,
    /// Generic parameter, index into the enclosing generics list.
    Param(u32, String),
    /// Inference variable. `int` marks an unsuffixed integer literal.
    Infer(u32, bool),
}

impl Ty {
    pub fn is_copy(&self, adt_is_copy: &dyn Fn(DefId) -> bool) -> bool {
        match self {
            Ty::Int(_) | Ty::F32 | Ty::F64 | Ty::Bool | Ty::Unit | Ty::Never | Ty::Ptr(_) | Ty::Fn(..) => true,
            Ty::Clos(..) | Ty::SelfTy => false,
            Ty::Dyn { mutable, .. } => !mutable,
            Ty::Ref { mutable, .. } => !mutable,
            Ty::Adt(d, _) => adt_is_copy(*d),
            Ty::Param(..) | Ty::Infer(..) => false,
        }
    }
    pub fn is_int(&self) -> bool {
        matches!(self, Ty::Int(_) | Ty::Infer(_, true))
    }
    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::Int(_) | Ty::F32 | Ty::F64 | Ty::Infer(_, true))
    }
    pub fn is_scalar(&self) -> bool {
        matches!(self, Ty::Int(_) | Ty::F32 | Ty::F64 | Ty::Bool | Ty::Ref { .. } | Ty::Ptr(_) | Ty::Fn(..))
    }
    pub fn is_unit(&self) -> bool {
        matches!(self, Ty::Unit | Ty::Never)
    }
    pub fn has_ref(&self) -> bool {
        match self {
            Ty::Ref { .. } => true,
            Ty::Adt(_, args) => args.iter().any(|a| a.has_ref()),
            Ty::Clos(..) => true,
            Ty::Fn(..) | Ty::Ptr(_) => false,
            _ => false,
        }
    }
    pub fn has_infer(&self) -> bool {
        match self {
            Ty::Infer(..) => true,
            Ty::Ref { inner, .. } | Ty::Ptr(inner) => inner.has_infer(),
            Ty::Adt(_, args) => args.iter().any(|a| a.has_infer()),
            Ty::Fn(ps, r) | Ty::Clos(ps, r) => ps.iter().any(|a| a.has_infer()) || r.has_infer(),
            _ => false,
        }
    }
    pub fn has_param(&self) -> bool {
        match self {
            Ty::Param(..) => true,
            Ty::Ref { inner, .. } | Ty::Ptr(inner) => inner.has_param(),
            Ty::Adt(_, args) => args.iter().any(|a| a.has_param()),
            Ty::Fn(ps, r) | Ty::Clos(ps, r) => ps.iter().any(|a| a.has_param()) || r.has_param(),
            _ => false,
        }
    }
    /// Replace `Param(i)` with `args[i]`.
    pub fn subst(&self, args: &[Ty]) -> Ty {
        match self {
            Ty::Param(i, _) => args.get(*i as usize).cloned().unwrap_or_else(|| self.clone()),
            Ty::Ref { mutable, lt, inner } => Ty::Ref { mutable: *mutable, lt: lt.clone(), inner: Box::new(inner.subst(args)) },
            Ty::Ptr(inner) => Ty::Ptr(Box::new(inner.subst(args))),
            Ty::Adt(d, a) => Ty::Adt(*d, a.iter().map(|t| t.subst(args)).collect()),
            Ty::Fn(ps, r) => Ty::Fn(ps.iter().map(|t| t.subst(args)).collect(), Box::new(r.subst(args))),
            Ty::Clos(ps, r) => Ty::Clos(ps.iter().map(|t| t.subst(args)).collect(), Box::new(r.subst(args))),
            Ty::Dyn { .. } | Ty::SelfTy => self.clone(),
            _ => self.clone(),
        }
    }
    pub fn deref_all(&self) -> &Ty {
        match self {
            Ty::Ref { inner, .. } => inner.deref_all(),
            t => t,
        }
    }
    /// Collects lifetime names appearing in the type.
    pub fn lifetimes(&self, out: &mut Vec<String>) {
        match self {
            Ty::Ref { lt, inner, .. } => {
                if let Some(l) = lt {
                    out.push(l.clone());
                }
                inner.lifetimes(out);
            }
            Ty::Adt(_, a) => a.iter().for_each(|t| t.lifetimes(out)),
            Ty::Ptr(i) => i.lifetimes(out),
            Ty::Fn(ps, r) | Ty::Clos(ps, r) => {
                ps.iter().for_each(|t| t.lifetimes(out));
                r.lifetimes(out);
            }
            _ => {}
        }
    }
}

/// Printer that knows ADT names; used in diagnostics.
pub struct TyPrinter<'a> {
    pub names: &'a dyn Fn(DefId) -> String,
}

impl<'a> TyPrinter<'a> {
    pub fn show(&self, t: &Ty) -> String {
        match t {
            Ty::Int(i) => i.name().to_string(),
            Ty::F32 => "f32".into(),
            Ty::F64 => "f64".into(),
            Ty::Bool => "bool".into(),
            Ty::Unit => "unit".into(),
            Ty::Never => "never".into(),
            Ty::Ref { mutable, lt, inner } => {
                let k = if *mutable { "&mut" } else { "&" };
                match lt {
                    Some(l) => format!("({} '{} {})", k, l, self.show(inner)),
                    None => format!("({} {})", k, self.show(inner)),
                }
            }
            Ty::Ptr(i) => format!("(ptr {})", self.show(i)),
            Ty::Adt(d, args) => {
                let n = (self.names)(*d);
                if args.is_empty() {
                    n
                } else {
                    let a: Vec<String> = args.iter().map(|t| self.show(t)).collect();
                    format!("({} {})", n, a.join(" "))
                }
            }
            Ty::Fn(ps, r) => fn_show("fn", ps, r, self),
            Ty::Clos(ps, r) => fn_show("clos", ps, r, self),
            Ty::Dyn { mutable, trait_id } => format!("({} (dyn {}))", if *mutable { "&mut" } else { "&" }, (self.names)(*trait_id)),
            Ty::SelfTy => "Self".into(),
            Ty::Param(_, n) => n.clone(),
            Ty::Infer(_, true) => "{int}".into(),
            Ty::Infer(_, false) => "?".into(),
        }
    }
}

fn fn_show(kind: &str, ps: &[Ty], r: &Ty, p: &TyPrinter) -> String {
    let a: Vec<String> = ps.iter().map(|t| p.show(t)).collect();
    format!("({} ({}) {})", kind, a.join(" "), p.show(r))
}

/// `(fn ...)` or `(clos ...)` from the same parameter and return types.
pub fn fn_clos(clos: bool, ps: Vec<Ty>, r: Ty) -> Ty {
    let r = Box::new(r);
    if clos { Ty::Clos(ps, r) } else { Ty::Fn(ps, r) }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p = TyPrinter { names: &|d: DefId| format!("adt{}", d.0) };
        write!(f, "{}", p.show(self))
    }
}
