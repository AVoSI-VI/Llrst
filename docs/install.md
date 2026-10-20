# Install

Requirements: Rust toolchain (`cargo`) and a C compiler used only as the linker.
On Linux or macOS that compiler is `cc`. On Windows it is `clang`, `gcc`, or `cl`.
Cranelift backends built in: x86_64, aarch64, riscv64, s390x.

```
scripts/install.sh            # Linux or macOS, installs to ~/.local/{bin,share/llrst}
scripts/install.sh /usr/local # or another prefix
scripts/install.ps1           # Windows, installs to %USERPROFILE%\.local
scripts/install.ps1 PREFIX
```

The script builds `llrc` in release mode, copies `std/` to `PREFIX/share/llrst/std`,
compiles the host runtime (`runtime/llr_stat.c` on Unix, `runtime/llr_win.c` and `runtime/llr_wsa.c` on Windows),
compiles every `tools/llr-*.llr`, and verifies with a hello world. Output is pipe records;
the last line is `done|prefix=...|tools=N`. On Windows, `llrc` compiles `llr_win.c` and `llr_wsa.c` itself
when the object is missing or older than the source.

`fs.mtime` and `fs.size` link that object. The C uses `st_mtime` and `st_size`, so the
layout is whatever the host headers define. On a Mac, the same object translates Linux
`open` flags, `sockaddr_in`, `EAGAIN`, and `RLIMIT_NOFILE`; `tests/run.sh` is the check.
`llrc` looks for the object in `$LLRST_RUNTIME`,
next to `$LLRST_STD`, `<exe>/../share/llrst/`, and `./runtime/`. Windows links
`llr_win` and `llr_wsa` from those same places.

`llrc` finds `std` in this order: `--std DIR`, `$LLRST_STD`, `<exe>/../std`,
`<exe>/../../std`, `<exe>/../share/llrst/std`, `./std`.

Cross compiling: `llrc build f.llr --target aarch64-unknown-linux-gnu --emit-obj` writes
`f.o`. `tests/cross.sh` links that object, plus riscv64 and s390x, with `lld` and
`-nostdlib`. Direct linking with `--target` uses `clang --target=...` and needs a
matching sysroot. apple-darwin is checked as an object and is not linked.

Windows host: `cargo build`, then `target\debug\llrc.exe run examples\hello.llr`.
`.cargo/config.toml` raises the Windows main-thread stack so the larger compiles fit.
`powershell -File tests\run.ps1` runs the golden suite. A Windows `--target` selects
`std/sys_windows.llr`. `llrc` links that program with `clang`, `gcc`, or `cl` and
`runtime/llr_win.c` plus `runtime/llr_wsa.c`. Output paths gain an `.exe` suffix. `/tmp` in programs maps to
the Windows temp directory.
