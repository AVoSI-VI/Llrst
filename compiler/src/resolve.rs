//! Module loading and item collection. Produces a `Program` with resolved signatures.
use crate::ast::{self, Item, TyExpr};
use crate::diag::{Diag, Diags, PResult, SourceMap, Span};
use crate::parse::parse_items;
use crate::sexpr::read_file;
use crate::ty::{DefId, IntTy, Ty};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct StructDef {
    pub generics: Vec<String>,
    pub lifetimes: Vec<String>,
    pub fields: Vec<(String, Ty)>,
}

pub struct EnumDef {
    pub generics: Vec<String>,
    pub lifetimes: Vec<String>,
    pub variants: Vec<(String, Vec<Ty>)>,
}

pub struct FnDef {
    pub generics: Vec<String>,
    pub lifetimes: Vec<String>,
    pub params: Vec<(String, Ty)>,
    pub ret: Ty,
    pub body: Vec<ast::Expr>,
    pub is_extern: bool,
    pub owner: Option<DefId>,
    pub has_body: bool,
}

pub struct ConstDef {
    pub ty: Ty,
    pub value: ast::Expr,
}

pub enum DefKind {
    Struct(StructDef),
    Enum(EnumDef),
    Fn(FnDef),
    Const(ConstDef),
}

pub struct Def {
    pub name: String,
    pub module: u32,
    pub span: Span,
    pub kind: DefKind,
}

pub struct Module {
    pub path: String,
    pub file: u32,
    pub items: HashMap<String, DefId>,
    pub uses: HashMap<String, u32>,
    pub tests: Vec<ast::TestDecl>,
    pub is_std: bool,
}

pub struct Program {
    pub defs: Vec<Def>,
    pub modules: Vec<Module>,
    pub methods: HashMap<DefId, HashMap<String, DefId>>,
    pub main_module: u32,
    pub prelude: HashMap<String, DefId>,
}

impl Program {
    pub fn def(&self, id: DefId) -> &Def {
        &self.defs[id.0 as usize]
    }
    pub fn fn_def(&self, id: DefId) -> &FnDef {
        match &self.def(id).kind {
            DefKind::Fn(f) => f,
            _ => panic!("not a fn"),
        }
    }
    pub fn struct_def(&self, id: DefId) -> Option<&StructDef> {
        match &self.def(id).kind {
            DefKind::Struct(s) => Some(s),
            _ => None,
        }
    }
    pub fn enum_def(&self, id: DefId) -> Option<&EnumDef> {
        match &self.def(id).kind {
            DefKind::Enum(e) => Some(e),
            _ => None,
        }
    }
    pub fn adt_generics(&self, id: DefId) -> &[String] {
        match &self.def(id).kind {
            DefKind::Struct(s) => &s.generics,
            DefKind::Enum(e) => &e.generics,
            _ => &[],
        }
    }
    pub fn type_name(&self, id: DefId) -> String {
        self.def(id).name.clone()
    }
    pub fn prelude_type(&self, name: &str) -> Option<DefId> {
        self.prelude.get(name).copied()
    }
    pub fn fn_path(&self, id: DefId) -> String {
        let d = self.def(id);
        match &d.kind {
            DefKind::Fn(f) => match f.owner {
                Some(o) => format!("{}.{}", self.def(o).name, d.name),
                None => format!("{}.{}", self.modules[d.module as usize].path.rsplit('.').next().unwrap_or(""), d.name),
            },
            _ => d.name.clone(),
        }
    }
    /// Resolve a (possibly dotted) type name in the scope of a module.
    pub fn lookup_type(&self, module: u32, name: &str) -> Option<DefId> {
        let m = &self.modules[module as usize];
        if let Some((a, b)) = name.split_once('.') {
            let mi = *m.uses.get(a)?;
            return self.modules[mi as usize].items.get(b).copied().filter(|d| self.is_type(*d));
        }
        if let Some(d) = m.items.get(name).copied().filter(|d| self.is_type(*d)) {
            return Some(d);
        }
        self.prelude.get(name).copied()
    }
    pub fn is_type(&self, d: DefId) -> bool {
        matches!(self.def(d).kind, DefKind::Struct(_) | DefKind::Enum(_))
    }
    /// Resolve a function path `name`, `mod.name`, `type.name`, `mod.type.name`.
    pub fn lookup_fn(&self, module: u32, path: &str) -> Option<DefId> {
        let m = &self.modules[module as usize];
        let parts: Vec<&str> = path.split('.').collect();
        match parts.len() {
            1 => m.items.get(parts[0]).copied().filter(|d| matches!(self.def(*d).kind, DefKind::Fn(_))),
            2 => {
                if let Some(&mi) = m.uses.get(parts[0]) {
                    if let Some(d) = self.modules[mi as usize].items.get(parts[1]) {
                        if matches!(self.def(*d).kind, DefKind::Fn(_)) {
                            return Some(*d);
                        }
                    }
                }
                let t = self.lookup_type(module, parts[0])?;
                if let Some(m) = self.methods.get(&t).and_then(|m| m.get(parts[1])) {
                    return Some(*m);
                }
                // Module-level fns of the module that defines the type: `vec.sort_str`.
                let tm = self.def(t).module;
                self.modules[tm as usize].items.get(parts[1]).copied().filter(|d| matches!(self.def(*d).kind, DefKind::Fn(_)))
            }
            3 => {
                let t = self.lookup_type(module, &format!("{}.{}", parts[0], parts[1]))?;
                self.methods.get(&t)?.get(parts[2]).copied()
            }
            _ => None,
        }
    }
    pub fn lookup_const(&self, module: u32, path: &str) -> Option<DefId> {
        let m = &self.modules[module as usize];
        let d = if let Some((a, b)) = path.split_once('.') {
            let mi = *m.uses.get(a)?;
            *self.modules[mi as usize].items.get(b)?
        } else {
            *m.items.get(path)?
        };
        matches!(self.def(d).kind, DefKind::Const(_)).then_some(d)
    }
}

pub struct Loader<'a> {
    pub sm: &'a mut SourceMap,
    pub diags: &'a Diags,
    pub std_dir: PathBuf,
    pub proj_dir: PathBuf,
    parsed: Vec<(String, u32, Vec<Item>, bool)>,
    by_path: HashMap<String, u32>,
}

impl<'a> Loader<'a> {
    pub fn new(sm: &'a mut SourceMap, diags: &'a Diags, std_dir: PathBuf, proj_dir: PathBuf) -> Self {
        Loader { sm, diags, std_dir, proj_dir, parsed: Vec::new(), by_path: HashMap::new() }
    }

    fn load_file(&mut self, mod_path: &str, file: &Path, is_std: bool) -> PResult<u32> {
        if let Some(&i) = self.by_path.get(mod_path) {
            return Ok(i);
        }
        let text = std::fs::read_to_string(file)
            .map_err(|_| Diag::error(Span::default(), "io.read").kv("path", file.display().to_string()))?;
        let fid = self.sm.add(file.display().to_string(), text);
        let forms = read_file(fid, self.sm.text(fid))?;
        let items = parse_items(&forms)?;
        let idx = self.parsed.len() as u32;
        self.by_path.insert(mod_path.to_string(), idx);
        self.parsed.push((mod_path.to_string(), fid, items, is_std));
        Ok(idx)
    }

    /// Loads the main file, all of std, and any `proj.*` modules reachable via `use`.
    pub fn load(mut self, main: &Path) -> PResult<Program> {
        let main_idx = self.load_file("main", main, false)?;
        let mut std_files: Vec<PathBuf> = std::fs::read_dir(&self.std_dir)
            .map_err(|_| Diag::error(Span::default(), "io.read").kv("path", self.std_dir.display().to_string()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map_or(false, |e| e == "llr"))
            .collect();
        std_files.sort();
        let main_canon = std::fs::canonicalize(main).ok();
        for f in std_files {
            let name = f.file_stem().unwrap().to_string_lossy().to_string();
            // Compiling a std file directly (e.g. its tests): it is both main and std.X.
            if std::fs::canonicalize(&f).ok() == main_canon {
                self.by_path.insert(format!("std.{}", name), main_idx);
                self.parsed[main_idx as usize].3 = true;
                continue;
            }
            self.load_file(&format!("std.{}", name), &f, true)?;
        }
        let mut i = 0;
        while i < self.parsed.len() {
            let uses: Vec<(Vec<String>, Span)> = self.parsed[i]
                .2
                .iter()
                .filter_map(|it| match it {
                    Item::Use(p, s) => Some((p.clone(), *s)),
                    _ => None,
                })
                .collect();
            for (p, sp) in uses {
                let path = p.join(".");
                if p.len() == 2 && p[0] == "proj" {
                    let f = self.proj_dir.join(format!("{}.llr", p[1]));
                    self.load_file(&path, &f, false).map_err(|d| if d.code == "io.read" { d.kv("use", path.clone()) } else { d })?;
                } else if !(p.len() == 2 && p[0] == "std" && self.by_path.contains_key(&path)) {
                    return Err(Diag::error(sp, "name.not_module").kv("name", path));
                }
            }
            i += 1;
        }
        collect(self.parsed, self.by_path, main_idx, self.diags)
    }
}

fn collect(parsed: Vec<(String, u32, Vec<Item>, bool)>, by_path: HashMap<String, u32>, main_idx: u32, diags: &Diags) -> PResult<Program> {
    let mut prog = Program { defs: Vec::new(), modules: Vec::new(), methods: HashMap::new(), main_module: main_idx, prelude: HashMap::new() };
    // Phase 1: modules, type names, uses.
    for (path, file, items, is_std) in &parsed {
        let mi = prog.modules.len() as u32;
        let mut m = Module { path: path.clone(), file: *file, items: HashMap::new(), uses: HashMap::new(), tests: Vec::new(), is_std: *is_std };
        for it in items {
            match it {
                Item::Use(p, _) => {
                    m.uses.insert(p.last().unwrap().clone(), by_path[&p.join(".")]);
                }
                Item::Struct(s) => {
                    let id = push_def(&mut prog, &mut m, &s.name, mi, s.span, DefKind::Struct(StructDef { generics: s.generics.types.clone(), lifetimes: s.generics.lifetimes.clone(), fields: Vec::new() }), diags);
                    if *is_std {
                        prog.prelude.insert(s.name.clone(), id);
                    }
                }
                Item::Enum(e) => {
                    let id = push_def(&mut prog, &mut m, &e.name, mi, e.span, DefKind::Enum(EnumDef { generics: e.generics.types.clone(), lifetimes: e.generics.lifetimes.clone(), variants: Vec::new() }), diags);
                    if *is_std {
                        prog.prelude.insert(e.name.clone(), id);
                    }
                }
                _ => {}
            }
        }
        prog.modules.push(m);
    }
    // Phase 2: field types, signatures, consts, methods.
    for (mi, (_, _, items, _)) in parsed.iter().enumerate() {
        let mi = mi as u32;
        for it in items {
            match it {
                Item::Struct(s) => {
                    let id = prog.modules[mi as usize].items[&s.name];
                    let fields = s.fields.iter().map(|f| Ok((f.name.clone(), resolve_ty(&prog, mi, &s.generics.types, &f.ty)?))).collect::<PResult<Vec<_>>>()?;
                    if let DefKind::Struct(sd) = &mut prog.defs[id.0 as usize].kind {
                        sd.fields = fields;
                    }
                }
                Item::Enum(e) => {
                    let id = prog.modules[mi as usize].items[&e.name];
                    let mut variants = Vec::new();
                    for v in &e.variants {
                        let tys = v.fields.iter().map(|t| resolve_ty(&prog, mi, &e.generics.types, t)).collect::<PResult<Vec<_>>>()?;
                        variants.push((v.name.clone(), tys));
                    }
                    if let DefKind::Enum(ed) = &mut prog.defs[id.0 as usize].kind {
                        ed.variants = variants;
                    }
                }
                Item::Fn(f) => {
                    let def = make_fn(&prog, mi, f, &[], None)?;
                    let mut m = std::mem::replace(&mut prog.modules[mi as usize], dummy_module());
                    push_def(&mut prog, &mut m, &f.name, mi, f.span, DefKind::Fn(def), diags);
                    prog.modules[mi as usize] = m;
                }
                Item::Impl(im) => {
                    let owner = prog.lookup_type(mi, &im.ty_name).ok_or_else(|| Diag::error(im.span, "name.not_type").kv("name", im.ty_name.clone()))?;
                    let owner_generics = prog.adt_generics(owner).to_vec();
                    if owner_generics.len() != im.generics.types.len() {
                        return Err(Diag::error(im.span, "type.generic_arity").kv("want", owner_generics.len().to_string()).kv("got", im.generics.types.len().to_string()));
                    }
                    for f in &im.fns {
                        let def = make_fn(&prog, mi, f, &im.generics.types, Some(owner))?;
                        let id = DefId(prog.defs.len() as u32);
                        let name = format!("{}.{}", im.ty_name, f.name);
                        prog.defs.push(Def { name: f.name.clone(), module: mi, span: f.span, kind: DefKind::Fn(def) });
                        let entry = prog.methods.entry(owner).or_default();
                        if entry.insert(f.name.clone(), id).is_some() {
                            diags.push(Diag::error(f.span, "name.dup").kv("name", name));
                        }
                    }
                }
                Item::Const(c) => {
                    let ty = resolve_ty(&prog, mi, &[], &c.ty)?;
                    let mut m = std::mem::replace(&mut prog.modules[mi as usize], dummy_module());
                    push_def(&mut prog, &mut m, &c.name, mi, c.span, DefKind::Const(ConstDef { ty, value: c.value.clone() }), diags);
                    prog.modules[mi as usize] = m;
                }
                Item::Test(t) => prog.modules[mi as usize].tests.push(t.clone()),
                Item::Use(..) | Item::Mod(..) => {}
            }
        }
    }
    Ok(prog)
}

fn dummy_module() -> Module {
    Module { path: String::new(), file: 0, items: HashMap::new(), uses: HashMap::new(), tests: Vec::new(), is_std: false }
}

fn push_def(prog: &mut Program, m: &mut Module, name: &str, module: u32, span: Span, kind: DefKind, diags: &Diags) -> DefId {
    let id = DefId(prog.defs.len() as u32);
    prog.defs.push(Def { name: name.to_string(), module, span, kind });
    if m.items.insert(name.to_string(), id).is_some() {
        diags.push(Diag::error(span, "name.dup").kv("name", name));
    }
    id
}

fn make_fn(prog: &Program, mi: u32, f: &ast::FnDecl, outer: &[String], owner: Option<DefId>) -> PResult<FnDef> {
    let mut generics = outer.to_vec();
    generics.extend(f.generics.types.iter().cloned());
    let params = f.params.iter().map(|p| Ok((p.name.clone(), resolve_ty(prog, mi, &generics, &p.ty)?))).collect::<PResult<Vec<_>>>()?;
    let ret = resolve_ty(prog, mi, &generics, &f.ret)?;
    Ok(FnDef {
        generics,
        lifetimes: f.generics.lifetimes.clone(),
        params,
        ret,
        body: f.body.clone(),
        is_extern: f.is_extern,
        owner,
        has_body: !f.is_extern,
    })
}

pub fn resolve_ty(prog: &Program, module: u32, generics: &[String], t: &TyExpr) -> PResult<Ty> {
    Ok(match t {
        TyExpr::Name(n, sp) => {
            if let Some(i) = IntTy::from_name(n) {
                return Ok(Ty::Int(i));
            }
            match n.as_str() {
                "f32" => return Ok(Ty::F32),
                "f64" => return Ok(Ty::F64),
                "bool" => return Ok(Ty::Bool),
                "unit" => return Ok(Ty::Unit),
                "never" => return Ok(Ty::Never),
                _ => {}
            }
            if let Some(i) = generics.iter().position(|g| g == n) {
                return Ok(Ty::Param(i as u32, n.clone()));
            }
            let d = prog.lookup_type(module, n).ok_or_else(|| Diag::error(*sp, "name.not_type").kv("name", n.clone()))?;
            let want = prog.adt_generics(d).len();
            if want != 0 {
                return Err(Diag::error(*sp, "type.generic_arity").kv("name", n.clone()).kv("want", want.to_string()).kv("got", "0"));
            }
            Ty::Adt(d, Vec::new())
        }
        TyExpr::App(n, args, sp) => {
            let d = prog.lookup_type(module, n).ok_or_else(|| Diag::error(*sp, "name.not_type").kv("name", n.clone()))?;
            let want = prog.adt_generics(d).len();
            if want != args.len() {
                return Err(Diag::error(*sp, "type.generic_arity").kv("name", n.clone()).kv("want", want.to_string()).kv("got", args.len().to_string()));
            }
            Ty::Adt(d, args.iter().map(|a| resolve_ty(prog, module, generics, a)).collect::<PResult<Vec<_>>>()?)
        }
        TyExpr::Ref { mutable, lt, inner, .. } => Ty::Ref { mutable: *mutable, lt: lt.clone(), inner: Box::new(resolve_ty(prog, module, generics, inner)?) },
        TyExpr::Ptr(inner, _) => Ty::Ptr(Box::new(resolve_ty(prog, module, generics, inner)?)),
        TyExpr::Fn(ps, r, _) => Ty::Fn(
            ps.iter().map(|a| resolve_ty(prog, module, generics, a)).collect::<PResult<Vec<_>>>()?,
            Box::new(resolve_ty(prog, module, generics, r)?),
        ),
    })
}
