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

`self/` accepts an empty triple, an `x86_64` triple that does not contain `apple`, and any
triple containing `aarch64` or `arm64`. `riscv64` and `s390x` stay on stage 0. `tests/cross.sh`
prints `skip|cross|target=...|self=x86-64 and aarch64 only` for those two when `LLRC` is a
`self/` compiler.

| triple | ISA | object |
|-------|-----|--------|
| empty | `uname -m` of `arm64` or `aarch64` is AArch64; otherwise x86-64 | Mach-O arm64 when the host is Darwin and the ISA is AArch64; COFF on Windows; otherwise ELF |
| `x86_64-unknown-linux-gnu` | x86-64 | ELF, `e_machine` 62 |
| `x86_64-pc-windows-gnu` | x86-64 | COFF |
| `aarch64-unknown-linux-gnu` | AArch64 | ELF, `e_machine` 183 |
| `aarch64-apple-darwin`, `arm64-apple-darwin` | AArch64 | Mach-O arm64 |
| `x86_64-apple-darwin` | | rejected |

An empty triple links with host `cc`. A Darwin host omits `-pie`. A non-empty Unix triple
links with `clang --target=<triple>`. Windows uses the probed linker in `link_win.llr`.
