//! Linking: write the object file and drive the system C compiler as the linker.
use crate::diag::{Diag, PResult, Span};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

#[derive(Clone)]
struct Tool {
    name: &'static str,
    msvc: bool,
    /// Extra environment from `vcvars64.bat` when `cl` is not already on `PATH`.
    env: Vec<(std::ffi::OsString, std::ffi::OsString)>,
}

pub fn host_exe(path: &Path, target: Option<&str>) -> PathBuf {
    if windows_os(target) {
        force_exe(path)
    } else {
        path.to_path_buf()
    }
}

pub fn link(obj: &[u8], out: &Path, target: Option<&str>, shared: bool) -> PResult<()> {
    if windows_os(target) {
        link_windows(obj, out, shared)
    } else {
        link_unix(obj, out, target, shared)
    }
}

fn windows_os(target: Option<&str>) -> bool {
    match target {
        None => cfg!(windows),
        Some(t) => t
            .parse::<target_lexicon::Triple>()
            .map(|tr| tr.operating_system == target_lexicon::OperatingSystem::Windows)
            .unwrap_or(false),
    }
}

fn force_exe(path: &Path) -> PathBuf {
    if matches!(path.extension().and_then(|e| e.to_str()), Some("exe") | Some("dll")) {
        return path.to_path_buf();
    }
    let mut name = path.as_os_str().to_owned();
    name.push(".exe");
    PathBuf::from(name)
}

fn push_raw(cmd: &mut Command, arg: &str) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.raw_arg(arg);
    }
    #[cfg(not(windows))]
    {
        cmd.arg(arg);
    }
}

/// `/Fo` and `/Fe` take the path glued on, with quotes only when it has spaces.
fn msvc_path(flag: &str, path: &Path) -> String {
    let text = path.display().to_string();
    if text.contains(' ') || text.contains('"') {
        format!("{flag}\"{text}\"")
    } else {
        format!("{flag}{text}")
    }
}

fn link_err(cmd: &Command, got: &str) -> Diag {
    Diag::error(Span::default(), "link.failed").kv("cmd", format!("{:?}", cmd)).kv("got", got.to_string())
}

fn wants_ui(obj: &[u8]) -> bool {
    let needle = b"llr_ui_open";
    obj.windows(needle.len()).any(|w| w == needle)
}

fn wants_web(obj: &[u8]) -> bool {
    let needle = b"llr_web_present";
    obj.windows(needle.len()).any(|w| w == needle)
}

fn ui_missing() -> Diag {
    Diag::error(Span::default(), "link.failed").kv("cmd", "cc").kv("got", "llr_ui_*.o missing")
}

fn ui_darwin(target: Option<&str>) -> bool {
    match target {
        Some(t) => t.contains("apple") || t.contains("darwin"),
        None => cfg!(target_os = "macos"),
    }
}

fn add_ui_unix(cmd: &mut Command, target: Option<&str>) -> PResult<()> {
    let darwin = ui_darwin(target);
    let (a, b) = if darwin { ("llr_ui_mac.o", "llr_ui_mac_in.o") } else { ("llr_ui_x11.o", "llr_ui_x11_in.o") };
    cmd.arg(find_named(a).ok_or_else(ui_missing)?);
    cmd.arg(find_named(b).ok_or_else(ui_missing)?);
    if darwin {
        cmd.arg("-framework").arg("Cocoa");
    } else {
        cmd.arg("-lX11");
    }
    Ok(())
}

fn wants_name(obj: &[u8], name: &str) -> bool {
    let needle = name.as_bytes();
    obj.windows(needle.len()).any(|w| w == needle)
}

fn link_unix(obj: &[u8], out: &Path, target: Option<&str>, shared: bool) -> PResult<()> {
    let obj_path = out.with_extension("o");
    write_obj(&obj_path, obj)?;
    let mut cmd = match target {
        Some(t) => {
            let mut c = Command::new("clang");
            c.arg(format!("--target={}", t));
            c
        }
        None => Command::new("cc"),
    };
    cmd.arg(&obj_path).arg("-o").arg(out);
    if let Some(shim) = find_named("llr_stat.o") {
        cmd.arg(shim);
    }
    if let Some(now) = find_named("llr_now.o") {
        cmd.arg(now);
    }
    if target.is_none() {
        if shared {
            cmd.arg("-shared");
        } else {
            // Apple clang rejects `-pie` as unused; PIE is already the default.
            #[cfg(not(target_os = "macos"))]
            cmd.arg("-pie");
        }
        cmd.arg("-pthread");
        // llr_walk.c declares these weak. Apple ld rejects an undefined weak symbol unless -U names it.
        #[cfg(target_os = "macos")]
        for sym in ["_llr_fnspans", "_llr_fnspans_n", "_llr_frame_walk"] {
            cmd.arg(format!("-Wl,-U,{sym}"));
        }
    } else if shared {
        cmd.arg("-shared");
    }
    if wants_ui(obj) {
        add_ui_unix(&mut cmd, target)?;
    }
    if wants_web(obj) {
        cmd.arg(find_named("llr_ui_web.o").ok_or_else(ui_missing)?);
    }
    if wants_name(obj, "llr_reload_") {
        cmd.arg(find_named("llr_reload.o").ok_or_else(|| {
            Diag::error(Span::default(), "link.failed").kv("cmd", "cc").kv("got", "llr_reload.o missing")
        })?);
    }
    if wants_name(obj, "llr_dlopen") {
        cmd.arg(find_named("llr_dl.o").ok_or_else(|| {
            Diag::error(Span::default(), "link.failed").kv("cmd", "cc").kv("got", "llr_dl.o missing")
        })?);
        #[cfg(not(target_os = "macos"))]
        cmd.arg("-ldl");
    }
    finish_link(&mut cmd, &obj_path)
}

fn link_windows(obj: &[u8], out: &Path, shared: bool) -> PResult<()> {
    let out = force_exe(out);
    let tool = cc_tool()?;
    let shim = ensure_shim(&tool)?;
    let ui = if wants_ui(obj) { ensure_ui(&tool)? } else { Vec::new() };
    let web = if wants_web(obj) { ensure_named(&tool, &["llr_ui_web.c"], "llr_ui_web.c")? } else { Vec::new() };
    let obj_path = out.with_extension(if tool.msvc { "obj" } else { "o" });
    write_obj(&obj_path, obj)?;
    let mut cmd = Command::new(tool.name);
    apply_env(&mut cmd, &tool);
    if tool.msvc {
        cmd.arg("/nologo");
        if shared {
            cmd.arg("/LD");
        }
        push_raw(&mut cmd, &msvc_path("/Fe:", &out));
        cmd.arg(&obj_path);
        for part in &shim {
            cmd.arg(part);
        }
        for part in &ui {
            cmd.arg(part);
        }
        for part in &web {
            cmd.arg(part);
        }
        if !ui.is_empty() {
            cmd.arg("user32.lib").arg("gdi32.lib").arg("imm32.lib").arg("shell32.lib");
            cmd.arg("ole32.lib").arg("oleaut32.lib").arg("uiautomationcore.lib");
        }
        cmd.arg("ws2_32.lib").arg("/link").arg("/STACK:8388608");
        if shared {
            cmd.arg("/EXPORT:llr_ui_boot")
                .arg("/EXPORT:llr_ui_frame")
                .arg("/EXPORT:llr_ui_drop")
                .arg("/EXPORT:llr_reload_app");
        }
    } else {
        if shared {
            cmd.arg("-shared");
        }
        cmd.arg(&obj_path);
        for part in &shim {
            cmd.arg(part);
        }
        for part in &ui {
            cmd.arg(part);
        }
        for part in &web {
            cmd.arg(part);
        }
        cmd.arg("-o").arg(&out).arg("-lws2_32");
        if !ui.is_empty() {
            cmd.args(["-luser32", "-lgdi32", "-limm32", "-lshell32", "-lole32", "-loleaut32", "-luiautomationcore"]);
        }
        cmd.arg("-Wl,--stack,8388608");
    }
    finish_link(&mut cmd, &obj_path)
}

fn write_obj(path: &Path, obj: &[u8]) -> PResult<()> {
    std::fs::write(path, obj).map_err(|e| {
        Diag::error(Span::default(), "io.write").kv("path", path.display().to_string()).kv("got", e.to_string())
    })
}

fn finish_link(cmd: &mut Command, obj_path: &Path) -> PResult<()> {
    let status = cmd.status().map_err(|e| link_err(cmd, &e.to_string()))?;
    let _ = std::fs::remove_file(obj_path);
    if !status.success() {
        return Err(link_err(cmd, &status.code().unwrap_or(-1).to_string()));
    }
    Ok(())
}

fn cc_tool() -> PResult<Tool> {
    static CACHE: OnceLock<Option<Tool>> = OnceLock::new();
    CACHE
        .get_or_init(probe_cc)
        .clone()
        .ok_or_else(|| Diag::error(Span::default(), "link.failed").kv("cmd", "clang, gcc, or cl").kv("got", "missing"))
}

fn probe_cc() -> Option<Tool> {
    for name in ["clang", "gcc", "cc"] {
        if tool_ok(name, false) {
            return Some(Tool { name, msvc: false, env: Vec::new() });
        }
    }
    if tool_ok("cl", true) && std::env::var_os("INCLUDE").is_some() {
        return Some(Tool { name: "cl", msvc: true, env: Vec::new() });
    }
    capture_vcvars().map(|env| Tool { name: "cl", msvc: true, env })
}

fn apply_env(cmd: &mut Command, tool: &Tool) {
    for (key, val) in &tool.env {
        cmd.env(key, val);
    }
}

fn capture_vcvars() -> Option<Vec<(std::ffi::OsString, std::ffi::OsString)>> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let bat = find_vcvars()?;
        let script = format!("call \"{}\" >nul 2>&1 && set", bat.display());
        // `cmd /c` mishandles a quoted argument, so the script is passed raw.
        let mut cmd = Command::new("cmd");
        cmd.raw_arg("/c").raw_arg(&script);
        let out = cmd.output().ok()?;
        if !out.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let env: Vec<_> = text
            .lines()
            .filter_map(|line| line.split_once('='))
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        if env.is_empty() { None } else { Some(env) }
    }
    #[cfg(not(windows))]
    {
        None
    }
}

fn find_vcvars() -> Option<PathBuf> {
    let vswhere = PathBuf::from(r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe");
    if !vswhere.is_file() {
        return None;
    }
    let out = Command::new(&vswhere)
        .args(["-latest", "-products", "*", "-requires", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let install = String::from_utf8(out.stdout).ok()?;
    let bat = PathBuf::from(install.trim()).join("VC").join("Auxiliary").join("Build").join("vcvars64.bat");
    bat.is_file().then_some(bat)
}

fn tool_ok(name: &str, msvc: bool) -> bool {
    let mut cmd = Command::new(name);
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    if msvc {
        cmd.arg("/nologo").arg("/?");
    } else {
        cmd.arg("--version");
    }
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

fn ensure_ui(tool: &Tool) -> PResult<Vec<PathBuf>> {
    ensure_named(tool, &["llr_ui_win.c", "llr_ui_win_in.c", "llr_ui_win_a11y.c"], "llr_ui_win.c")
}

fn ensure_shim(tool: &Tool) -> PResult<Vec<PathBuf>> {
    ensure_named(
        tool,
        &["llr_wsa.c", "llr_win.c", "llr_win_sock.c", "llr_win_proc.c", "llr_now.c", "llr_reload.c", "llr_dl.c"],
        "llr_win.c",
    )
}

fn ensure_named(tool: &Tool, names: &[&str], anchor: &str) -> PResult<Vec<PathBuf>> {
    let ext = if tool.msvc { "obj" } else { "o" };
    if let Some(src) = find_named(anchor) {
        let dir = src.parent().unwrap_or_else(|| Path::new("."));
        let mut objs = Vec::new();
        for name in names {
            let file = dir.join(name);
            if !file.is_file() {
                let got = if anchor.starts_with("llr_ui") { "llr_ui_*.o missing".to_string() } else { format!("{name} missing") };
                return Err(Diag::error(Span::default(), "link.failed").kv("cmd", "clang, gcc, or cl").kv("got", got));
            }
            let obj = file.with_extension(ext);
            if stale(&file, &obj) {
                compile_shim(tool, &file, &obj)?;
            }
            objs.push(obj);
        }
        return Ok(objs);
    }
    let mut objs = Vec::new();
    for name in names {
        let stem = name.trim_end_matches(".c");
        let obj = find_named(&format!("{stem}.{ext}")).ok_or_else(|| {
            let got = if anchor.starts_with("llr_ui") { "llr_ui_*.o missing".to_string() } else { format!("{stem}.{ext} missing") };
            Diag::error(Span::default(), "link.failed").kv("cmd", "clang, gcc, or cl").kv("got", got)
        })?;
        objs.push(obj);
    }
    Ok(objs)
}

fn stale(src: &Path, obj: &Path) -> bool {
    let src_t = std::fs::metadata(src).and_then(|m| m.modified()).ok();
    let obj_t = std::fs::metadata(obj).and_then(|m| m.modified()).ok();
    match (src_t, obj_t) {
        (Some(s), Some(o)) => s > o,
        _ => true,
    }
}

fn compile_shim(tool: &Tool, src: &Path, obj: &Path) -> PResult<()> {
    let mut cmd = Command::new(tool.name);
    apply_env(&mut cmd, tool);
    if tool.msvc {
        cmd.arg("/nologo").arg("/std:c11").arg("/c").arg(src);
        push_raw(&mut cmd, &msvc_path("/Fo", obj));
    } else {
        cmd.arg("-c").arg(src).arg("-o").arg(obj);
    }
    let out = cmd.output().map_err(|e| link_err(&cmd, &e.to_string()))?;
    if out.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    let text = if err.trim().is_empty() { String::from_utf8_lossy(&out.stdout).into_owned() } else { err.into_owned() };
    let line = text
        .lines()
        .find(|l| l.contains("error"))
        .or_else(|| text.lines().find(|l| !l.trim().is_empty()))
        .unwrap_or("compile failed");
    Err(link_err(&cmd, line))
}

fn find_named(name: &str) -> Option<PathBuf> {
    let mut cands = Vec::new();
    if let Ok(dir) = std::env::var("LLRST_RUNTIME") {
        cands.push(PathBuf::from(dir).join(name));
    }
    if let Ok(std_dir) = std::env::var("LLRST_STD") {
        if let Some(parent) = Path::new(&std_dir).parent() {
            cands.push(parent.join(name));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            cands.push(dir.join("../share/llrst").join(name));
            cands.push(dir.join("../../runtime").join(name));
        }
    }
    cands.push(PathBuf::from("runtime").join(name));
    cands.into_iter().find(|p| p.is_file())
}
