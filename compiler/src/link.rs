//! Linking: write the object file and drive the system C compiler as the linker.
use crate::diag::{Diag, PResult, Span};
use std::path::Path;
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
