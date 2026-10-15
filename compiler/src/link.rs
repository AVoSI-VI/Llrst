//! Linking: write the object file and drive the system C compiler as the linker.
use crate::diag::{Diag, PResult, Span};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn link(obj: &[u8], out: &Path, target: Option<&str>) -> PResult<()> {
    let obj_path = out.with_extension("o");
    std::fs::write(&obj_path, obj).map_err(|e| Diag::error(Span::default(), "io.write").kv("path", obj_path.display().to_string()).kv("got", e.to_string()))?;
    let mut cmd = match target {
        Some(t) => {
            let mut c = Command::new("clang");
            c.arg(format!("--target={}", t));
            c
        }
        None => Command::new("cc"),
    };
    cmd.arg(&obj_path).arg("-o").arg(out);
    if let Some(shim) = find_shim() {
        cmd.arg(shim);
    }
    if target.is_none() {
        cmd.arg("-pie");
    }
    let status = cmd.status().map_err(|e| Diag::error(Span::default(), "link.failed").kv("cmd", format!("{:?}", cmd)).kv("got", e.to_string()))?;
    let _ = std::fs::remove_file(&obj_path);
    if !status.success() {
        return Err(Diag::error(Span::default(), "link.failed").kv("cmd", format!("{:?}", cmd)).kv("status", status.code().unwrap_or(-1).to_string()));
    }
    Ok(())
}

/// The host stat object, if one has been compiled. Absent is fine for programs that
/// never call `fs.mtime` or `fs.size`; those calls then fail at link time.
fn find_shim() -> Option<PathBuf> {
    let mut cands = Vec::new();
    if let Ok(dir) = std::env::var("LLRST_RUNTIME") {
        cands.push(PathBuf::from(dir).join("llr_stat.o"));
    }
    if let Ok(std_dir) = std::env::var("LLRST_STD") {
        if let Some(parent) = Path::new(&std_dir).parent() {
            cands.push(parent.join("llr_stat.o"));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            cands.push(dir.join("../share/llrst/llr_stat.o"));
        }
    }
    cands.push(PathBuf::from("runtime/llr_stat.o"));
    cands.into_iter().find(|p| p.is_file())
}
