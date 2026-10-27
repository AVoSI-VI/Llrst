# Install

Requirements: a Rust toolchain (`cargo`) for stage 0, and a C compiler used only as the linker.
On Linux or macOS that compiler is `cc`. On Windows it is `cl`.
Stage 0 is the Rust compiler. The installed `llrc` is stage-2 `llrs`, which emits x86-64.

```
scripts/install.sh            # Linux or macOS, installs to ~/.local/{bin,share/llrst}
scripts/install.sh /usr/local # or another prefix
scripts/install.ps1           # Windows, installs to %USERPROFILE%\.local
scripts/install.ps1 PREFIX
```

The script builds release `llrc` as stage 0 and does not install that binary. Stage 0 builds
`llrs0`, `llrs0` builds `llrs1`, and `llrs1` builds `llrs2`. `llrs1` and `llrs2` must match
byte for byte. The script then installs `llrs2` as `llrc`, copies `std/` to
`PREFIX/share/llrst/std`, compiles the host runtime (`runtime/llr_stat.c` on Unix,
`runtime/llr_win.c` and `runtime/llr_wsa.c` on Windows), compiles every `tools/llr-*.llr`,
and verifies with a hello world. Output is pipe records; the last line is
`done|prefix=...|tools=N`. The installed compiler links those prebuilt objects. It does not
compile the C files when the object is missing or older.

`fs.mtime` and `fs.size` link that object. The C uses `st_mtime` and `st_size`, so the
layout is whatever the host headers define. On a Mac, the same object translates Linux
`open` flags, `sockaddr_in`, `EAGAIN`, and `RLIMIT_NOFILE`. The installed compiler does not
target Darwin; that gap is in HANDOFF section 8. `tests/run.sh` is the check for stage 0.
The compiler looks for the object in `$LLRST_RUNTIME`, next to `$LLRST_STD`,
`<exe>/../share/llrst/`, `<exe>/../../runtime/`, and `./runtime/`. Windows links `llr_win`
and `llr_wsa` from those same places.

`llrc` finds `std` in this order: `--std DIR`, `$LLRST_STD`, `<exe>/../std`,
`<exe>/../../std`, `<exe>/../share/llrst/std`, `./std`. The executable path comes from
`llr_exe_path`.

Windows host: `scripts/install.ps1` builds stage 0, then the three `llrs` stages, and
installs `llrc.exe`. `.cargo/config.toml` raises the Windows main-thread stack so the
stage-0 compiles fit. The installed compiler links with `cl` and `/STACK:8388608`.
`powershell -File tests\run.ps1` runs the golden suite. Output paths gain an `.exe` suffix.
`/tmp` in programs maps to the Windows temp directory.
