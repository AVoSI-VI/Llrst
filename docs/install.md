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

The script builds release `llrc` as stage 0. On Linux it does not install that binary.
Stage 0 builds `llrs0`, `llrs0` builds `llrs1`, and `llrs1` builds `llrs2`. `llrs1` and
`llrs2` must match byte for byte. `scripts/self.sh` is that bootstrap on Unix. On Darwin,
`self/` emits ELF and Apple `ld` reports `unknown file type`, so the script installs stage 0
as `llrc`. That binary emits the host Mach-O. `scripts/self.ps1` is the same
bootstrap on Windows; it sets `SOURCE_DATE_EPOCH=0` for the compare, and the link line keeps
`/Brepro`. `--stage` stops after `self|stage2=equal`. On Linux and Windows the install
script then installs `llrs2` as `llrc`. It copies `std/` and `examples/` to
`PREFIX/share/llrst/`, compiles the host runtime
(`runtime/llr_stat.c` on Unix, `runtime/llr_wsa.c`, `runtime/llr_win.c`, `runtime/llr_win_sock.c`,
and `runtime/llr_win_proc.c` on Windows), compiles every `tools/llr-*.llr`, and verifies with
`examples/hello.llr`. Output is pipe records; the last line is `done|prefix=...|tools=N`.
The installed compiler links those prebuilt objects. It does not compile the C files when
the object is missing or older.

`fs.mtime` and `fs.size` link that object. The C uses `st_mtime` and `st_size`, so the
layout is whatever the host headers define. On a Mac, the same object translates Linux
`open` flags, `sockaddr_in`, `EAGAIN`, and `RLIMIT_NOFILE`, and the compiler renames
`open`, `fcntl`, `bind`, `connect`, `socket`, and `setrlimit` to the `llr_*` definitions
in that object. A Darwin host link omits `-pie` because Apple clang rejects it. A non-empty
Unix `--target` links with `clang --target=<triple>` and does not pass `-pie` or `-pthread`.
`aarch64`, `riscv64`, `s390x`, and `apple-darwin` objects stay on stage 0. `tests/run.sh` and
`tests/run.ps1` check whichever `llrc` `LLRC` names; stage 0 is `target/release/llrc`. Both
runners also call `tests/check`, `tests/mir`, `tests/self`, and `tests/cross` against that
stage 0 binary. The compiler looks for the object in `$LLRST_RUNTIME`, next to `$LLRST_STD`,
`<exe>/../share/llrst/`, `<exe>/../../runtime/`, and `./runtime/`. Windows links `llr_wsa`,
`llr_win`, `llr_win_sock`, and `llr_win_proc` from those same places, using the first of
`clang`, `gcc`, `cc`, and `cl` whose prebuilt shim extension (`.o` or `.obj`) is present.

`llrc` finds `std` in this order: `--std DIR`, `$LLRST_STD`, `<exe>/../std`,
`<exe>/../../std`, `<exe>/../share/llrst/std`, `./std`. The executable path comes from
`llr_exe_path`.

Windows host: `scripts/install.ps1` builds stage 0, then `scripts/self.ps1` builds the three
`llrs` stages, and installs `llrc.exe`. `.cargo/config.toml` raises the Windows main-thread
stack so the stage-0 compiles fit. The installed compiler links with `cl` when the `.obj`
shims are the ones on disk, and passes `/STACK:8388608`. `powershell -File tests\run.ps1`
runs the golden suite. Output paths gain an `.exe` suffix. `/tmp` in programs maps to the
Windows temp directory. `llrc run` passes the argument vector through `llr_exec`.

## Commands

Flags may come before or after the subcommand. `version` prints `llrc|0.1.0`.

```
usage|llrc build FILE [-o OUT] [--target TRIPLE] [--std DIR] [--test] [--emit-obj] [--checked] [--deny CODE]
usage|llrc run FILE [--std DIR] [--deny CODE] [-- ARGS...]
usage|llrc check FILE [--std DIR] [--deny CODE]
usage|llrc emit-mir FILE [--std DIR] [--deny CODE]
usage|llrc version
```

`--emit-obj` writes the object and does not link. `--checked` traps wrapping arithmetic,
out-of-range shifts, and casts that do not fit. `--deny CODE` may be repeated; each named
warning is printed as `E` and the process exits 1. `run FILE -- ARGS` passes `ARGS` to the
program. Stage 0 sets Cranelift `opt_level=speed`. `llrs` keeps every local in a stack slot
and does not run that optimizer.
