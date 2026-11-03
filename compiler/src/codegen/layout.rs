//! Memory layout of types: sizes, alignment, field offsets, enum tag/payload placement.
use crate::resolve::{DefKind, Program};
use crate::ty::{IntTy, Ty};
use cranelift_codegen::ir::types;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Layout {
    pub size: u32,
    pub align: u32,
    /// Struct field offsets, or per-variant field offsets for enums.
    pub fields: Vec<u32>,
    pub variants: Vec<Vec<u32>>,
}

pub struct Layouts<'a> {
    pub prog: &'a Program,
    cache: HashMap<Ty, Layout>,
}

pub const PTR_BYTES: u32 = 8;

fn align_up(x: u32, a: u32) -> u32 {
    if a == 0 {
        x
    } else {
        (x + a - 1) / a * a
    }
}

impl<'a> Layouts<'a> {
    pub fn new(prog: &'a Program) -> Self {
        Layouts { prog, cache: HashMap::new() }
    }

    pub fn of(&mut self, t: &Ty) -> Layout {
        if let Some(l) = self.cache.get(t) {
            return l.clone();
        }
        let l = self.compute(t);
        self.cache.insert(t.clone(), l.clone());
        l
    }

    pub fn size(&mut self, t: &Ty) -> u32 {
        self.of(t).size
    }

    fn compute(&mut self, t: &Ty) -> Layout {
        let scalar = |n: u32| Layout { size: n, align: n, fields: Vec::new(), variants: Vec::new() };
        match t {
            Ty::Int(i) => scalar(i.bytes()),
            Ty::F32 => scalar(4),
            Ty::F64 => scalar(8),
            Ty::Bool => scalar(1),
            Ty::Unit | Ty::Never => Layout { size: 0, align: 1, fields: Vec::new(), variants: Vec::new() },
            Ty::Ref { .. } | Ty::Ptr(_) | Ty::Fn(..) => scalar(PTR_BYTES),
            Ty::Clos(..) => Layout { size: PTR_BYTES * 3, align: PTR_BYTES, fields: vec![0, PTR_BYTES, PTR_BYTES * 2], variants: Vec::new() },
            Ty::Dyn { .. } | Ty::BoxDyn { .. } => Layout { size: PTR_BYTES * 2, align: PTR_BYTES, fields: vec![0, PTR_BYTES], variants: Vec::new() },
            Ty::SelfTy => panic!("layout of Self"),
            Ty::Adt(d, args) => match &self.prog.def(*d).kind {
                DefKind::Struct(s) => {
                    let tys: Vec<Ty> = s.fields.iter().map(|f| f.1.subst(args)).collect();
                    let (size, align, fields) = self.seq(&tys, 0);
                    Layout { size, align, fields, variants: Vec::new() }
                }
                DefKind::Enum(e) => {
                    let mut align = 4u32;
                    let mut size = 4u32;
                    let mut variants = Vec::new();
                    let payload_align = e
                        .variants
                        .iter()
                        .flat_map(|v| v.1.iter())
                        .map(|f| self.of(&f.subst(args)).align)
                        .max()
                        .unwrap_or(1);
                    let payload_off = align_up(4, payload_align);
                    for v in &e.variants {
                        let tys: Vec<Ty> = v.1.iter().map(|f| f.subst(args)).collect();
                        let (vs, va, offs) = self.seq(&tys, payload_off);
                        size = size.max(vs);
                        align = align.max(va);
                        variants.push(offs);
                    }
                    Layout { size: align_up(size, align), align, fields: Vec::new(), variants }
                }
                _ => unreachable!(),
            },
            Ty::Param(..) | Ty::Infer(..) => panic!("layout of unresolved type {}", t),
        }
    }

    /// Lays out `tys` sequentially starting at `start`. Returns (end size, align, offsets).
    pub fn seq(&mut self, tys: &[Ty], start: u32) -> (u32, u32, Vec<u32>) {
        let mut off = start;
        let mut align = 1;
        let mut offs = Vec::new();
        for t in tys {
            let l = self.of(t);
            off = align_up(off, l.align);
            offs.push(off);
            off += l.size;
            align = align.max(l.align);
        }
        (align_up(off, align), align, offs)
    }

    /// Byte offset of field `i` in variant `v` of an enum type.
    pub fn variant_field_offset(&mut self, t: &Ty, v: u32, i: u32) -> u32 {
        self.of(t).variants[v as usize][i as usize]
    }
}

/// Cranelift type for a scalar Llrst type.
pub fn clif_type(t: &Ty) -> Option<types::Type> {
    Some(match t {
        Ty::Int(i) => match i {
            IntTy::I8 | IntTy::U8 => types::I8,
            IntTy::I16 | IntTy::U16 => types::I16,
            IntTy::I32 | IntTy::U32 => types::I32,
            IntTy::I64 | IntTy::U64 => types::I64,
        },
        Ty::F32 => types::F32,
        Ty::F64 => types::F64,
        Ty::Bool => types::I8,
        Ty::Ref { .. } | Ty::Ptr(_) | Ty::Fn(..) => types::I64,
        _ => return None,
    })
}
