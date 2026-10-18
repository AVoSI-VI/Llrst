//! Diagnostics: structured, one line each, pipe separated. See docs/spec/errors.md.
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Write as _;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub file: u32,
    pub lo: u32,
    pub hi: u32,
}

impl Span {
    pub fn new(file: u32, lo: usize, hi: usize) -> Span {
        Span { file, lo: lo as u32, hi: hi as u32 }
    }
    pub fn to(self, other: Span) -> Span {
        Span { file: self.file, lo: self.lo.min(other.lo), hi: self.hi.max(other.hi) }
    }
}

pub struct SourceFile {
    pub path: String,
    pub text: String,
    line_starts: Vec<u32>,
}

#[derive(Default)]
pub struct SourceMap {
    pub files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn add(&mut self, path: String, text: String) -> u32 {
        let mut line_starts = vec![0u32];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        self.files.push(SourceFile { path, text, line_starts });
        (self.files.len() - 1) as u32
    }
    /// 1-based line and column of a byte offset.
    pub fn line_col(&self, span: Span) -> (u32, u32) {
        let f = &self.files[span.file as usize];
        let line = match f.line_starts.binary_search(&span.lo) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        (line as u32 + 1, span.lo - f.line_starts[line] + 1)
    }
    pub fn path(&self, file: u32) -> &str {
        &self.files[file as usize].path
    }
    pub fn text(&self, file: u32) -> &str {
        &self.files[file as usize].text
    }
    /// Owned copy of paths and line starts, for codegen after the map is no longer borrowed.
    pub fn export_locs(&self) -> Vec<(String, Vec<u32>)> {
        self.files.iter().map(|f| (f.path.clone(), f.line_starts.clone())).collect()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
}

#[derive(Clone)]
pub struct Diag {
    pub level: Level,
    pub span: Span,
    pub code: &'static str,
    pub kv: Vec<(&'static str, String)>,
}

impl Diag {
    pub fn error(span: Span, code: &'static str) -> Diag {
        Diag { level: Level::Error, span, code, kv: Vec::new() }
    }
    pub fn warn(span: Span, code: &'static str) -> Diag {
        Diag { level: Level::Warn, span, code, kv: Vec::new() }
    }
    pub fn kv(mut self, k: &'static str, v: impl Into<String>) -> Diag {
        self.kv.push((k, v.into()));
        self
    }
    pub fn render(&self, sm: &SourceMap) -> String {
        let (line, col) = sm.line_col(self.span);
        let mut s = String::new();
        let tag = if self.level == Level::Error { "E" } else { "W" };
        let _ = write!(
            s,
            "{}|{}|{}|{}|{}|{}",
            tag,
            sm.path(self.span.file),
            line,
            col,
            self.span.hi.saturating_sub(self.span.lo),
            self.code
        );
        for (k, v) in &self.kv {
            let _ = write!(s, "|{}={}", k, escape(v));
        }
        s
    }
}

pub fn escape(v: &str) -> String {
    v.replace('|', "\\x7c").replace('\n', "\\n")
}

/// Shared diagnostic sink. Passes push into it and the driver decides when to stop.
#[derive(Default)]
pub struct Diags {
    items: RefCell<Vec<Diag>>,
    deny: RefCell<HashSet<String>>,
}

impl Diags {
    pub fn set_deny(&self, codes: &[String]) {
        *self.deny.borrow_mut() = codes.iter().cloned().collect();
    }
    pub fn push(&self, mut d: Diag) {
        if d.level == Level::Warn && self.deny.borrow().contains(d.code) {
            d.level = Level::Error;
        }
        self.items.borrow_mut().push(d);
    }
    pub fn err(&self, span: Span, code: &'static str) -> Diag {
        Diag::error(span, code)
    }
    pub fn has_errors(&self) -> bool {
        self.items.borrow().iter().any(|d| d.level == Level::Error)
    }
    pub fn error_count(&self) -> usize {
        self.items.borrow().iter().filter(|d| d.level == Level::Error).count()
    }
    pub fn take(&self) -> Vec<Diag> {
        std::mem::take(&mut *self.items.borrow_mut())
    }
    pub fn flush(&self, sm: &SourceMap) {
        let mut items = self.take();
        items.sort_by_key(|d| (d.span.file, d.span.lo));
        for d in items {
            eprintln!("{}", d.render(sm));
        }
    }
}

/// Result alias for passes that bail on the first structural error.
pub type PResult<T> = Result<T, Diag>;
