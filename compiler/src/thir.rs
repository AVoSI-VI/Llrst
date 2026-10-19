//! Typed IR produced by the checker for one monomorphic function instance.
use crate::ast::{BinOp, Lit};
use crate::diag::Span;
use crate::ty::{DefId, Ty};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalId(pub u32);

#[derive(Clone, Debug)]
pub struct Local {
    pub name: String,
    pub ty: Ty,
    pub mutable: bool,
    pub span: Span,
}

/// Fully checked body of one function instance.
pub struct FnBody {
    pub def: DefId,
    pub targs: Vec<Ty>,
    pub locals: Vec<Local>,
    pub params: Vec<LocalId>,
    pub ret: Ty,
    pub body: Expr,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    PtrNull,
    PtrAdd,
    PtrRead,
    PtrWrite,
    PtrEq,
    MemReplace,
    Argc,
    Argv,
    Arch,
    Drop,
    /// Bitwise copy of a value regardless of type; std internals only.
    Forget,
}

impl Builtin {
    pub fn from_path(p: &str) -> Option<Builtin> {
        Some(match p {
            "ptr.null" => Builtin::PtrNull,
            "ptr.add" => Builtin::PtrAdd,
            "ptr.read" => Builtin::PtrRead,
            "ptr.write" => Builtin::PtrWrite,
            "ptr.eq" => Builtin::PtrEq,
            "mem.replace" => Builtin::MemReplace,
            "mem.forget" => Builtin::Forget,
            "sys.argc" => Builtin::Argc,
            "sys.argv" => Builtin::Argv,
            "sys.arch" => Builtin::Arch,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
pub enum Callee {
    Fn(DefId, Vec<Ty>),
    Ptr(Box<Expr>),
    Builtin(Builtin),
    Sizeof(Ty),
    Clos(Box<Expr>),
}

#[derive(Clone, Debug)]
pub enum Pat {
    Wild,
    Bind(LocalId),
    Lit(Lit),
    Variant { idx: u32, subs: Vec<Pat> },
}

#[derive(Clone, Debug)]
pub struct Arm {
    pub pat: Pat,
    pub body: Expr,
    pub span: Span,
}

/// A closure lowered as its own function: arguments are the captures, then the user parameters.
#[derive(Clone, Debug)]
pub struct ClosBody {
    pub locals: Vec<Local>,
    pub params: Vec<LocalId>,
    pub ret: Ty,
    pub body: Box<Expr>,
    pub caps: Vec<Ty>,
    pub user: Vec<Ty>,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Lit(Lit),
    Local(LocalId),
    Const(DefId),
    Let { local: LocalId, init: Option<Box<Expr>> },
    Assign { place: Box<Expr>, value: Box<Expr> },
    Block(Vec<Expr>),
    If { cond: Box<Expr>, then: Box<Expr>, els: Option<Box<Expr>> },
    While { cond: Box<Expr>, body: Box<Expr> },
    Loop(Box<Expr>),
    For { local: LocalId, lo: Box<Expr>, hi: Box<Expr>, body: Box<Expr> },
    Break,
    Continue,
    Return(Option<Box<Expr>>),
    /// `by_ref` is Some(mutable) when the scrutinee was a reference to an enum.
    Match { scrut: Box<Expr>, arms: Vec<Arm>, by_ref: Option<bool> },
    Ref { mutable: bool, place: Box<Expr> },
    Deref(Box<Expr>),
    Field { base: Box<Expr>, idx: u32 },
    Cast(Box<Expr>),
    Bin { op: BinOp, l: Box<Expr>, r: Box<Expr> },
    Neg(Box<Expr>),
    WNeg(Box<Expr>),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Call { callee: Callee, args: Vec<Expr> },
    Struct { def: DefId, fields: Vec<Expr> },
    Variant { def: DefId, idx: u32, fields: Vec<Expr> },
    FnRef(DefId),
    /// `(try e)`: `is_res` selects res vs opt.
    Try { expr: Box<Expr>, is_res: bool },
    /// Owned closure. `caps` are the enclosing bindings, copied or moved into the env.
    Closure { caps: Vec<Expr>, inner: ClosBody },
}

impl Expr {
    pub fn is_place(&self) -> bool {
        match &self.kind {
            ExprKind::Local(_) | ExprKind::Deref(_) => true,
            ExprKind::Field { base, .. } => base.is_place(),
            _ => false,
        }
    }
}
