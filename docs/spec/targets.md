# Targets

`--target TRIPLE` selects the ISA and the object format. An empty triple is the host.
`target.unsupported` rejects a triple the compiler does not emit.

## Stage 0

Stage 0 (`compiler/`, Cranelift) accepts a triple Cranelift can lower. `tests/cross.sh`
checks these:

| triple | object |
|-------|--------|
| `aarch64-unknown-linux-gnu` | ELF, then a freestanding `ld.lld` link |
| `riscv64gc-unknown-linux-gnu` | ELF, freestanding `ld.lld` link |
| `s390x-unknown-linux-gnu` | ELF, freestanding `ld.lld` link |
| `aarch64-apple-darwin` | Mach-O arm64; `file` checks the object |
| `x86_64-pc-windows-gnu` | MIR only in the suite (`sys.os_id` is 2) |

An empty triple uses the host ISA.

## self/

`self/` accepts an empty triple, any `x86_64` triple, any triple containing `riscv64`, and
any triple containing `aarch64` or `arm64`. `s390x` stays on stage 0. `tests/cross.sh`
prints `skip|cross|target=...|self=no s390x` for that triple when `LLRC` is a `self/` compiler.

| triple | ISA | object |
|-------|-----|--------|
| empty | `uname -m` of `arm64` or `aarch64` is AArch64; otherwise x86-64 | Mach-O of that ISA when the host is Darwin; COFF on Windows; otherwise ELF |
| `x86_64-unknown-linux-gnu` | x86-64 | ELF, `e_machine` 62 |
| `x86_64-pc-windows-gnu` | x86-64 | COFF |
| `x86_64-apple-darwin` | x86-64, SysV | Mach-O x86-64 |
| `riscv64gc-unknown-linux-gnu` | RV64GC, LP64D | ELF, `e_machine` 243, float ABI double |
| `aarch64-unknown-linux-gnu` | AArch64 | ELF, `e_machine` 183 |
| `aarch64-apple-darwin`, `arm64-apple-darwin` | AArch64 | Mach-O arm64 |
| `aarch64-apple-ios`, `arm64-apple-ios` | AArch64 | Mach-O arm64 |
| `aarch64-apple-ios-simulator`, `arm64-apple-ios-simulator` | AArch64 | Mach-O arm64, `iphonesimulator` SDK |
| `aarch64-linux-android` | AArch64 | ELF shared library, `e_machine` 183 |

An empty triple links with host `cc`. A Darwin host omits `-pie`. A non-empty Unix triple
links with `clang --target=<triple>` and does not pass `-pie` or `-pthread`. An Apple
triple links with `xcrun clang -target arm64-apple-macos`, `x86_64-apple-macos`, or
`arm64-apple-ios` and `-isysroot` from `xcrun --sdk macosx --show-sdk-path` or
`xcrun --sdk iphoneos --show-sdk-path`. An Android triple links with
`$ANDROID_NDK/toolchains/llvm/prebuilt/<host>/bin/aarch64-linux-android21-clang -shared`.
That link is a NativeActivity library: `llr_stat_android.o`, `llr_now_android.o`,
`android_native_app_glue` (`llr_ui_android_glue.o`), `llr_android_main.o`, and
`-landroid -llog -u ANativeActivity_onCreate`. `android_main` calls `main`. A reference
to `llr_ui_open` also links `llr_ui_android.o` and `llr_ui_android_in.o`. An iOS triple
links `llr_stat_ios.o` and `llr_now_ios.o` (or the `_ios_sim` pair), renames `_main` to
`_llr_user_main`, and links `llr_ios_main.o` with UIKit. A reference to `llr_ui_open`
also links the UI pair. The simulator triple uses `-target arm64-apple-ios-simulator`
and the `iphonesimulator` SDK. A missing
SDK, NDK, or target object is `link.failed`. Windows uses the probed linker in `link_win.llr`.
`--debug` writes line tables from the statement spans. ELF and Mach-O gain a DWARF 5
`.debug_line` section (`__debug_line` in `__DWARF` on Mach-O). Each function is one
sequence: `DW_LNE_set_address` relocates to that function's symbol, then one row per
statement (`line`, `column`, file). The sequence ends one byte past the function.
COFF writes the same lines into `.text`'s line-number table. `wasm32` ignores the flag.
Without `--debug` the object bytes are unchanged.

`$LLRST_CC` may be `cl`, `clang`, `gcc`, or `cc`. When it is set, that driver is used and the
other compiler probes are skipped. `cl` still runs `vcvars64.bat` when `cl` is not already
on `PATH`. The environment that batch file prints is not cached.

`tests/a64.sh` builds each `tests/run` golden with `--target aarch64-unknown-linux-gnu`,
links it freestanding, and runs it. A Linux aarch64 host runs the binary. Any other host
uses `qemu-aarch64`. A missing emulator prints `fail|setup|qemu=missing`. The script is
not part of `tests/run.sh`.

## wasm32

`self/` also accepts `wasm32-wasi` and `wasm32-unknown-unknown`. The object is one Wasm
module (default path ends in `.wasm`). `llrc run` on either triple is `target.unsupported`;
`tests/wasm.sh` runs the module with `wasmtime`. Language pointers, `str` headers, and
vtable words stay 8 bytes, little-endian, the same layout as x86-64. A wasm address is the
low 32 bits of that pointer.

`load.llr` selects `sys_wasm.llr` when the triple contains `wasm32`, before the Windows or
POSIX file. `sys.os_id` is `3`. `wasm32-wasi` imports WASI preview1. `wasm32-unknown-unknown`
imports those same names from the module `llr`, then the canvas imports `present`, `size`,
`scale`, `now_ms`, `request_frame`, `poll_event`, `set_cursor`, `clipboard_get`,
`clipboard_set`, `text_input`, `quit`, `bind`, `app`, and `history`. Its `_start` calls boot and `main`
and returns, so the page can keep calling the exported `llr_frame`. A WASI module still calls
`proc_exit` and does not import `present`. There is no network, no threads, no shell, and
no directory listing: those externs write `target.unsupported` and return `-1` or null.
