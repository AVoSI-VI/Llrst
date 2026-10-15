# Install

Requirements: Rust toolchain (`cargo`), a C compiler (`cc`, used only as the linker),
Linux or macOS. Cranelift backends built in: x86_64, aarch64, riscv64, s390x.

```
scripts/install.sh            # installs to ~/.local/{bin,share/llrst}
scripts/install.sh /usr/local # or another prefix
```

The script builds `llrc` in release mode, copies `std/` to `PREFIX/share/llrst/std`,
compiles every `tools/llr-*.llr`, and verifies with a hello world. Output is pipe records;
the last line is `done|prefix=...|tools=N`.

`llrc` finds `std` in this order: `--std DIR`, `$LLRST_STD`, `<exe>/../std`,
`<exe>/../../std`, `<exe>/../share/llrst/std`, `./std`.

Cross compiling: `llrc build f.llr --target aarch64-unknown-linux-gnu --emit-obj` writes
`f.o`; link it on the target with `cc f.o -o f`. Direct linking with `--target` uses
`clang --target=...` and needs a matching sysroot.

Windows: not supported by the runtime (`std/sys.llr` binds POSIX libc).
