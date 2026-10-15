//! Surface AST, produced by `parse` from S-expressions.
use crate::diag::Span;

#[derive(Clone, Debug)]
pub enum TyExpr {
    Name(String, Span),                       // i64, str, point, T, geo.point
    App(String, Vec<TyExpr>, Span),           // (vec T) (res A B) (geo.pair A B)
    Ref { mutable: bool, lt: Option<String>, inner: Box<TyExpr>, span: Span },
    Ptr(Box<TyExpr>, Span),
    Fn(Vec<TyExpr>, Box<TyExpr>, Span),
}

impl TyExpr {
    pub fn span(&self) -> Span {
        match self {
            TyExpr::Name(_, s) | TyExpr::App(_, _, s) | TyExpr::Ptr(_, s) | TyExpr::Fn(_, _, s) => *s,
            TyExpr::Ref { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub ty: TyExpr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Generics {
    pub types: Vec<String>,
    pub lifetimes: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct FnDecl {
    pub name: String,
    pub generics: Generics,
    pub params: Vec<Param>,
    pub ret: TyExpr,
    pub body: Vec<Expr>,
    pub is_extern: bool,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub ty: TyExpr,
}

#[derive(Clone, Debug)]
pub struct StructDecl {
    pub name: String,
    pub generics: Generics,
    pub fields: Vec<Field>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<TyExpr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumDecl {
    pub name: String,
    pub generics: Generics,
    pub variants: Vec<Variant>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ImplDecl {
    pub ty_name: String,
    pub generics: Generics,
    pub fns: Vec<FnDecl>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ConstDecl {
    pub name: String,
    pub ty: TyExpr,
    pub value: Expr,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TestDecl {
    pub name: String,
    pub body: Vec<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Item {
    Use(Vec<String>, Span),
    Mod(String, Span),
    Fn(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Impl(ImplDecl),
    Const(ConstDecl),
    Test(TestDecl),
}

#[derive(Clone, Debug)]
pub enum Lit {
    Int(i128, Option<String>), // value, suffix
    Float(f64, Option<String>),
    Bool(bool),
    Str(Vec<u8>),
    Unit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem, Eq, Ne, Lt, Le, Gt, Ge, BitAnd, BitOr, BitXor, Shl, Shr,
    /// Always wrap, even under `llrc --checked`.
    WAdd, WSub, WMul,
}

impl BinOp {
    pub fn from_str(s: &str) -> Option<BinOp> {
        Some(match s {
            "+" => BinOp::Add, "-" => BinOp::Sub, "*" => BinOp::Mul, "/" => BinOp::Div, "%" => BinOp::Rem,
            "==" => BinOp::Eq, "!=" => BinOp::Ne, "<" => BinOp::Lt, "<=" => BinOp::Le, ">" => BinOp::Gt, ">=" => BinOp::Ge,
            "band" => BinOp::BitAnd, "bor" => BinOp::BitOr, "bxor" => BinOp::BitXor, "shl" => BinOp::Shl, "shr" => BinOp::Shr,
            "wadd" => BinOp::WAdd, "wsub" => BinOp::WSub, "wmul" => BinOp::WMul,
            _ => return None,
        })
    }
    pub fn is_cmp(self) -> bool {
        matches!(self, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge)
    }
}

#[derive(Clone, Debug)]
pub enum Pat {
    Wild(Span),
    Bind(String, Span),
    Lit(Lit, Span),
    Variant { path: String, subs: Vec<Pat>, span: Span },
}

impl Pat {
    pub fn span(&self) -> Span {
        match self {
            Pat::Wild(s) | Pat::Bind(_, s) | Pat::Lit(_, s) => *s,
            Pat::Variant { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Arm {
    pub pat: Pat,
    pub body: Expr,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Lit(Lit),
    Name(String),
    Let { name: String, mutable: bool, ty: Option<TyExpr>, init: Box<Expr> },
    Set { place: Box<Expr>, value: Box<Expr> },
    Do(Vec<Expr>),
    If { cond: Box<Expr>, then: Box<Expr>, els: Option<Box<Expr>> },
    While { cond: Box<Expr>, body: Vec<Expr> },
    Loop(Vec<Expr>),
    For { name: String, lo: Box<Expr>, hi: Box<Expr>, body: Vec<Expr> },
    Break,
    Continue,
    Return(Option<Box<Expr>>),
    Match { scrut: Box<Expr>, arms: Vec<Arm> },
    Ref { mutable: bool, place: Box<Expr> },
    Deref(Box<Expr>),
    Field { base: Box<Expr>, name: String },
    Cast { ty: TyExpr, expr: Box<Expr> },
    Bin { op: BinOp, l: Box<Expr>, r: Box<Expr> },
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Neg(Box<Expr>),
    /// Negation that always wraps.
    WNeg(Box<Expr>),
    Call { callee: String, targs: Vec<TyExpr>, args: Vec<Expr> },
    CallPtr { callee: Box<Expr>, args: Vec<Expr> },
    Try(Box<Expr>),
    Panic(Box<Expr>),
    Assert(Box<Expr>),
    AssertEq(Box<Expr>, Box<Expr>),
    Sizeof(TyExpr),
    Drop(Box<Expr>),
}
