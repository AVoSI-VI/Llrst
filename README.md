# Llrst

A compiled, memory-safe language designed for models, not people. Files end in `.llr`.
The syntax is S-expressions with three delimiters, the memory model is Rust's (ownership,
borrows, NLL), the compiler `llrc` is Rust on Cranelift, and every diagnostic is one
machine-readable line.

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
| `compiler/` | `llrc` (Rust) |
| `std/` | standard library (Llrst) |
| `tools/` | `llr-read llr-grep llr-find llr-edit llr-write llr-ls llr-rm llr-git llr-http llr-fmt llr-test llr-diff` (Llrst) |
| `tests/` | golden tests; `tests/run.sh` |

Install: `scripts/install.sh` on Linux or macOS, `scripts/install.ps1` on Windows (see `docs/install.md`). Build only: `cargo build --release`.
Try it: `llrc run examples/hello.llr`.

License: MIT (see `LICENSE`).
