# Llrst

Llrst 0.1.0. A compiled, memory-safe language designed for models, not people. Files end in `.llr`.
The syntax is S-expressions with three delimiters, the memory model is Rust's (ownership,
borrows, NLL), and every diagnostic is one machine-readable line. The installed compiler
`llrc` is stage-2 `llrs` from `self/`. Stage 0 in `compiler/` is Rust on Cranelift.
`cargo build --release` produces that stage-0 binary and does not install it.

```
(use std.io)
(fn main () unit
  (io.puts "hello world"))
```

| path | content |
|------|---------|
| `HANDOFF.md` | rules, decisions, architecture, implementation notes, phased work plan; finished plans in `history/` |
| `examples/` | runnable programs; start with `examples/hello.llr` |
| `docs/spec/` | grammar, tokens, types, memory model, error format |
| `docs/canon/` | the one right way per task |
| `docs/std/` | standard library reference |
| `docs/tools/` | agent tools written in Llrst |
| `docs/workflows/` | step lists for agents |
| `compiler/` | stage 0 `llrc` (Rust, Cranelift); not installed |
| `self/` | installed compiler (`llrs`); the `llrc` command is stage-2 `llrs` |
| `std/` | standard library (Llrst) |
| `runtime/` | C shims: `llr_stat.c` on Unix; `llr_wsa.c`, `llr_win.c`, `llr_win_sock.c`, `llr_win_proc.c` on Windows |
| `scripts/` | `install.sh`, `install.ps1`, `self.sh`, `check-lines.sh` |
| `tools/` | `llr-read llr-grep llr-find llr-edit llr-write llr-ls llr-rm llr-git llr-http llr-fmt llr-test llr-diff llr-doc llr-check` (Llrst) |
| `tests/` | golden tests; `tests/run.sh` |

Install: `scripts/install.sh` on Linux or macOS, `scripts/install.ps1` on Windows (see `docs/install.md`). `cargo build --release` produces stage 0 only.
Try it: `llrc run examples/hello.llr`.

License: MIT (see `LICENSE`).
