//! Load main, sorted std, the host OS externs, then `proj.*` and `ui.*` on use.
use crate::ast::Item;
use crate::diag::{Diag, Diags, PResult, SourceMap, Span};
use crate::parse::parse_items;
use crate::resolve::{collect, Program};
use crate::sexpr::read_file;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Loader<'a> {
    pub sm: &'a mut SourceMap,
    pub diags: &'a Diags,
    pub std_dir: PathBuf,
    pub proj_dir: PathBuf,
    pub ui_dir: Option<PathBuf>,
    parsed: Vec<(String, u32, Vec<Item>, bool)>,
    by_path: HashMap<String, u32>,
}

impl<'a> Loader<'a> {
    pub fn new(sm: &'a mut SourceMap, diags: &'a Diags, std_dir: PathBuf, proj_dir: PathBuf, ui_dir: Option<PathBuf>) -> Self {
        Loader { sm, diags, std_dir, proj_dir, ui_dir, parsed: Vec::new(), by_path: HashMap::new() }
    }

    fn load_file(&mut self, mod_path: &str, file: &Path, is_std: bool) -> PResult<u32> {
        if let Some(&i) = self.by_path.get(mod_path) {
            return Ok(i);
        }
        let text = std::fs::read_to_string(file).map_err(|_| {
            Diag::error(Span::default(), "io.read").kv("path", file.display().to_string().replace('\\', "/"))
        })?;
        let shown = file.display().to_string().replace('\\', "/");
        let fid = self.sm.add(shown, text);
        let forms = read_file(fid, self.sm.text(fid))?;
        let items = parse_items(&forms)?;
        let idx = self.parsed.len() as u32;
        self.by_path.insert(mod_path.to_string(), idx);
        self.parsed.push((mod_path.to_string(), fid, items, is_std));
        Ok(idx)
    }

    pub fn load(mut self, main: &Path, target: Option<&str>) -> PResult<Program> {
        let os_file = crate::driver::os_extern_file(target)?;
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
            if name == "sys_posix" || name == "sys_windows" || name == "sys_wasm" {
                continue;
            }
            if std::fs::canonicalize(&f).ok() == main_canon {
                self.by_path.insert(format!("std.{}", name), main_idx);
                self.parsed[main_idx as usize].0 = format!("std.{}", name);
                self.parsed[main_idx as usize].3 = true;
                continue;
            }
            self.load_file(&format!("std.{}", name), &f, true)?;
        }
        self.append_os(os_file)?;
        let mut i = 0;
        while i < self.parsed.len() {
            let uses: Vec<(Vec<String>, Span)> = self.parsed[i]
                .2
                .iter()
                .filter_map(|it| match it {
                    Item::Use(p, s, _) => Some((p.clone(), *s)),
                    _ => None,
                })
                .collect();
            for (p, sp) in uses {
                let path = p.join(".");
                if p.len() >= 2 && p[0] == "proj" {
                    let f = proj_file(&self.proj_dir, &p[1..]);
                    self.load_file(&path, &f, false).map_err(|d| if d.code == "io.read" { d.kv("use", path.clone()) } else { d })?;
                } else if p.len() == 2 && p[0] == "ui" {
                    let dir = self.ui_dir.clone().ok_or_else(|| {
                        Diag::error(Span::default(), "io.read").kv("path", "ui").kv("fix", "set LLRST_UI")
                    })?;
                    let f = dir.join(format!("{}.llr", p[1]));
                    self.load_file(&path, &f, false).map_err(|d| if d.code == "io.read" { d.kv("use", path.clone()) } else { d })?;
                } else if !(p.len() == 2 && p[0] == "std" && self.by_path.contains_key(&path)) {
                    return Err(Diag::error(sp, "name.not_module").kv("name", path));
                }
            }
            i += 1;
        }
        collect(self.parsed, self.by_path, main_idx, self.diags)
    }

    fn append_os(&mut self, name: &str) -> PResult<()> {
        let file = self.std_dir.join(name);
        let text = std::fs::read_to_string(&file)
            .map_err(|_| Diag::error(Span::default(), "io.read").kv("path", file.display().to_string()))?;
        let shown = file.display().to_string().replace('\\', "/");
        let fid = self.sm.add(shown, text);
        let extra = parse_items(&read_file(fid, self.sm.text(fid))?)?;
        let idx = *self.by_path.get("std.sys").ok_or_else(|| Diag::error(Span::default(), "io.read").kv("path", "std/sys.llr"))?;
        self.parsed[idx as usize].2.extend(extra);
        Ok(())
    }
}

/// `proj.a.b` is `a/b.llr` under the main file's directory. `proj.name` stays `name.llr`.
fn proj_file(dir: &Path, segs: &[String]) -> PathBuf {
    let mut path = dir.to_path_buf();
    let last = segs.len().saturating_sub(1);
    for (i, seg) in segs.iter().enumerate() {
        if i == last {
            path.push(format!("{seg}.llr"));
        } else {
            path.push(seg);
        }
    }
    path
}
