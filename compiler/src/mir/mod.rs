//! Mid-level IR: a control-flow graph of basic blocks over places and operands.
pub mod lint;
pub mod lower;
mod lower_call;
mod lower_div;
mod lower_match;
pub mod print;

use crate::ast::BinOp;
use crate::diag::Span;
use crate::resolve::{DefKind, Program};
use crate::ty::{DefId, IntTy, Ty};

pub type BlockId = u32;
pub type LocalIdx = u32;

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub ty: Ty,
    pub name: String,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Proj {
    Field(u32),
    Deref,
    Downcast(u32),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Place {
    pub local: LocalIdx,
    pub proj: Vec<Proj>,
}

impl Place {
    pub fn local(l: LocalIdx) -> Place {
        Place { local: l, proj: Vec::new() }
    }
    pub fn field(mut self, i: u32) -> Place {
        self.proj.push(Proj::Field(i));
        self
    }
    pub fn deref(mut self) -> Place {
        self.proj.push(Proj::Deref);
        self
    }
    pub fn downcast(mut self, v: u32) -> Place {
        self.proj.push(Proj::Downcast(v));
        self
    }
    pub fn is_local(&self) -> bool {
        self.proj.is_empty()
    }
    /// True when the place passes through a dereference (its storage is not the local).
    pub fn has_deref(&self) -> bool {
        self.proj.iter().any(|p| matches!(p, Proj::Deref))
    }
}

#[derive(Clone, Debug)]
pub enum Const {
    Int(i128, IntTy),
    Float(f64, bool),
    Bool(bool),
    Str(Vec<u8>),
    Unit,
    Fn(DefId, Vec<Ty>),
}

#[derive(Clone, Debug)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Const(Const, Ty),
}

impl Operand {
    pub fn place(&self) -> Option<&Place> {
        match self {
            Operand::Copy(p) | Operand::Move(p) => Some(p),
            Operand::Const(..) => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Rvalue {
    Use(Operand),
    Ref { mutable: bool, place: Place, two_phase: bool },
    Bin(BinOp, Operand, Operand),
    Neg(Operand),
    WNeg(Operand),
    Not(Operand),
    Cast(Operand, Ty),
    Struct(Ty, Vec<Operand>),
    Variant(Ty, u32, Vec<Operand>),
    Discr(Place),
    PtrNull,
    PtrAdd(Operand, Operand, Ty),
    PtrEq(Operand, Operand),
    Argc,
    Argv,
    Sizeof(Ty),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Assign(Place, Rvalue),
    /// Drop the value if its drop flag is set (whole locals) or unconditionally (projections).
    Drop(Place),
    /// Clear the drop flag without running drop glue.
    Forget(LocalIdx),
    StorageDead(LocalIdx),
    Nop,
}

#[derive(Clone, Debug)]
pub enum Func {
    Def(DefId, Vec<Ty>),
    Ptr(Operand),
}

#[derive(Clone, Debug)]
pub enum Term {
    Goto(BlockId),
    If(Operand, BlockId, BlockId),
    Switch(Operand, Vec<(u128, BlockId)>, BlockId),
    Call { func: Func, args: Vec<Operand>, dest: Place, next: BlockId, span: Span },
    Return,
    Unreachable,
    /// Placeholder while a block is being built.
    None,
}

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub stmts: Vec<(Stmt, Span)>,
    pub term: Term,
    pub term_span: Span,
}

pub struct Body {
    pub name: String,
    pub locals: Vec<LocalDecl>,
    pub arg_count: usize,
    pub blocks: Vec<BasicBlock>,
    pub span: Span,
    /// Named lifetime parameters, for cross-function region rules.
    pub lifetimes: Vec<String>,
}

impl Body {
    pub fn local_ty(&self, l: LocalIdx) -> &Ty {
        &self.locals[l as usize].ty
    }
    pub fn successors(&self, b: BlockId) -> Vec<BlockId> {
        match &self.blocks[b as usize].term {
            Term::Goto(t) => vec![*t],
            Term::If(_, a, b) => vec![*a, *b],
            Term::Switch(_, cases, d) => {
                let mut v: Vec<BlockId> = cases.iter().map(|c| c.1).collect();
                v.push(*d);
                v
            }
            Term::Call { next, .. } => vec![*next],
            Term::Return | Term::Unreachable | Term::None => vec![],
        }
    }
}

/// Type of a place, following projections.
pub fn place_ty(prog: &Program, body: &Body, p: &Place) -> Ty {
    let mut t = body.local_ty(p.local).clone();
    let mut variant: Option<u32> = None;
    for pr in &p.proj {
        match pr {
            Proj::Deref => {
                t = match t {
                    Ty::Ref { inner, .. } | Ty::Ptr(inner) => *inner,
                    other => panic!("deref of {}", other),
                };
            }
            Proj::Downcast(v) => variant = Some(*v),
            Proj::Field(i) => {
                let Ty::Adt(d, args) = &t else { panic!("field of {}", t) };
                let ft = match (&prog.def(*d).kind, variant) {
                    (DefKind::Struct(s), _) => s.fields[*i as usize].1.clone(),
                    (DefKind::Enum(e), Some(v)) => e.variants[v as usize].1[*i as usize].clone(),
                    _ => panic!("bad field projection"),
                };
                t = ft.subst(args);
                variant = None;
            }
        }
    }
    t
}

pub fn is_copy(prog: &Program, t: &Ty) -> bool {
    let _ = prog;
    t.is_copy(&|_| false)
}

/// Whether values of this type run code when dropped.
pub fn needs_drop(prog: &Program, t: &Ty) -> bool {
    match t {
        Ty::Adt(d, args) => {
            if prog.methods.get(d).map_or(false, |m| m.contains_key("drop")) {
                return true;
            }
            match &prog.def(*d).kind {
                DefKind::Struct(s) => s.fields.iter().any(|f| needs_drop(prog, &f.1.subst(args))),
                DefKind::Enum(e) => e.variants.iter().any(|v| v.1.iter().any(|f| needs_drop(prog, &f.subst(args)))),
                _ => false,
            }
        }
        _ => false,
    }
}

/// The user-defined `drop` method of an ADT, if any.
pub fn drop_method(prog: &Program, t: &Ty) -> Option<DefId> {
    match t {
        Ty::Adt(d, _) => prog.methods.get(d).and_then(|m| m.get("drop")).copied(),
        _ => None,
    }
}
