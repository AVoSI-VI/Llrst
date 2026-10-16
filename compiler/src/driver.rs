//! Compilation driver: load, check, lower, borrow-check, codegen, link.
use crate::borrow::borrowck;
use crate::check::{check_fn, check_test};
use crate::codegen::{make_isa, Codegen, Instance};
use crate::diag::{Diag, Diags, PResult, SourceMap, Span};
use crate::mir::lower::lower_fn;
use crate::mir::print::print_body;
use crate::resolve::{DefKind, Loader, Program};
use crate::ty::{DefId, Ty};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct Options {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub target: Option<String>,
    pub std_dir: Option<PathBuf>,
    pub test: bool,
    pub emit_obj: bool,
    pub emit_mir: bool,
    pub check_only: bool,
    pub checked: bool,
}

pub fn find_std(opt: &Options) -> PResult<PathBuf> {
    if let Some(d) = &opt.std_dir {
        return Ok(d.clone());
    }
    if let Ok(d) = std::env::var("LLRST_STD") {
        return Ok(PathBuf::from(d));
    }
    let mut cands = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            cands.push(dir.join("../std"));
            cands.push(dir.join("../../std"));
            cands.push(dir.join("../share/llrst/std"));
        }
    }
    cands.push(PathBuf::from("std"));
    for c in cands {
        if c.join("sys.llr").exists() {
            return Ok(c);
        }
    }
    Err(Diag::error(Span::default(), "io.read").kv("path", "std").kv("fix", "set LLRST_STD"))
}

/// Runs the pipeline. Returns the process exit code.
pub fn run(opt: Options) -> i32 {
    let mut sm = SourceMap::default();
    let diags = Diags::default();
    let code = match pipeline(&opt, &mut sm, &diags) {
        Ok(()) if !diags.has_errors() => 0,
        Ok(()) => 1,
        Err(d) => {
            diags.push(d);
            1
        }
    };
    diags.flush(&sm);
    code
}

fn pipeline(opt: &Options, sm: &mut SourceMap, diags: &Diags) -> PResult<()> {
    let std_dir = find_std(opt)?;
    let proj_dir = opt.input.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
    let mut prog = Loader::new(sm, diags, std_dir, proj_dir).load(&opt.input)?;
    if diags.has_errors() {
        return Ok(());
    }
    let sm: &SourceMap = sm;
    let isa = make_isa(opt.target.as_deref())?;
    prog.target_arch = isa.triple().architecture.to_string();
    let mut cg = Codegen::new(&prog, isa, sm, opt.checked)?;

    // Always check every non-generic function of user modules so errors surface even in
    // unreachable code.
    let mut user_fns: Vec<DefId> = Vec::new();
    for (i, d) in prog.defs.iter().enumerate() {
        if prog.modules[d.module as usize].is_std {
            continue;
        }
        if let DefKind::Fn(f) = &d.kind {
            if f.generics.is_empty() && !f.is_extern {
                user_fns.push(DefId(i as u32));
            }
        }
    }
    let main_def = prog.modules[prog.main_module as usize].items.get("main").copied();

    let mut st = Pipeline { prog: &prog, diags, sm, opt, seen: HashSet::new(), mir_dump: String::new() };

    for f in &user_fns {
        if let Err(d) = st.process(&(*f, Vec::new()), &mut cg, false) {
            diags.push(d);
        }
    }
    if diags.has_errors() || opt.check_only {
        if opt.emit_mir {
            print!("{}", st.mir_dump);
        }
        return Ok(());
    }
    // From here on, everything reachable is compiled. User fns were checked but not
    // compiled; reset `seen` so reachable ones get code.
    st.seen.clear();

    let entry = if opt.test {
        let tests = prog.modules[prog.main_module as usize].tests.clone();
        let mut test_fns = Vec::new();
        for (i, t) in tests.iter().enumerate() {
            let body = check_test(&prog, diags, sm, prog.main_module, t)?;
            let mir = lower_fn(&prog, diags, sm, &body, format!("test.{}", t.name), Vec::new(), opt.checked);
            if opt.emit_mir {
                st.mir_dump.push_str(&print_body(&mir));
            }
            borrowck(&prog, diags, sm, &mir);
            crate::mir::lint::lint(diags, &mir);
            if diags.has_errors() {
                continue;
            }
            let id = cg.declare_test_fn(i);
            cg.compile_body_with_id(&mir, id)?;
            test_fns.push((t.name.clone(), id));
        }
        let run = std_fn(&prog, "test", "run")?;
        let done = std_fn(&prog, "test", "done")?;
        let run_id = cg.func_id(&(run, Vec::new()));
        let done_id = cg.func_id(&(done, Vec::new()));
        cg.emit_test_entry(&test_fns, run_id, done_id)?
    } else {
        let main = main_def.ok_or_else(|| Diag::error(Span::default(), "name.unknown").kv("name", "main").kv("fix", "(fn main () unit ...)"))?;
        let fd = prog.fn_def(main);
        if !fd.params.is_empty() || fd.ret != Ty::Unit {
            return Err(Diag::error(prog.def(main).span, "type.mismatch").kv("want", "(fn main () unit)").kv("got", "other signature"));
        }
        cg.func_id(&(main, Vec::new()))
    };
    // Mono loop.
    loop {
        while let Some(inst) = cg.discovered.pop() {
            if let Err(d) = st.process(&inst, &mut cg, true) {
                diags.push(d);
            }
            if diags.has_errors() {
                break;
            }
        }
        if diags.has_errors() {
            break;
        }
        cg.finish_drops()?;
        if cg.discovered.is_empty() {
            break;
        }
    }
    if opt.emit_mir {
        print!("{}", st.mir_dump);
    }
    if diags.has_errors() {
        return Ok(());
    }
    cg.emit_entry(entry)?;
    let obj = cg.emit_object()?;
    let out = opt.output.clone().unwrap_or_else(|| opt.input.with_extension(""));
    if opt.emit_obj {
        let p = out.with_extension("o");
        std::fs::write(&p, &obj).map_err(|e| Diag::error(Span::default(), "io.write").kv("path", p.display().to_string()).kv("got", e.to_string()))?;
        return Ok(());
    }
    crate::link::link(&obj, &out, opt.target.as_deref())
}

fn std_fn(prog: &Program, module: &str, name: &str) -> PResult<DefId> {
    let path = format!("std.{}", module);
    prog.modules
        .iter()
        .find(|m| m.path == path)
        .and_then(|m| m.items.get(name).copied())
        .ok_or_else(|| Diag::error(Span::default(), "name.unknown").kv("name", format!("{}.{}", module, name)))
}

struct Pipeline<'a> {
    prog: &'a Program,
    diags: &'a Diags,
    sm: &'a SourceMap,
    opt: &'a Options,
    seen: HashSet<Instance>,
    mir_dump: String,
}

impl<'a> Pipeline<'a> {
    /// Checks, lowers and borrow-checks one instance; compiles it when `codegen` is set.
    fn process(&mut self, inst: &Instance, cg: &mut Codegen, codegen: bool) -> PResult<()> {
        if !self.seen.insert(inst.clone()) {
            return Ok(());
        }
        let body = check_fn(self.prog, self.diags, self.sm, inst.0, &inst.1)?;
        let name = self.prog.fn_path(inst.0);
        let lifetimes = self.prog.fn_def(inst.0).lifetimes.clone();
        let mir = lower_fn(self.prog, self.diags, self.sm, &body, name, lifetimes, self.opt.checked);
        if self.opt.emit_mir {
            self.mir_dump.push_str(&print_body(&mir));
        }
        borrowck(self.prog, self.diags, self.sm, &mir);
        crate::mir::lint::lint(self.diags, &mir);
        if codegen && !self.diags.has_errors() {
            cg.compile_body(&mir, inst)?;
        }
        Ok(())
    }
}
