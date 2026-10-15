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
| `HANDOFF.md` | status, design decisions, implementation notes, known limits, next steps |
| `docs/spec/` | grammar, tokens, types, memory model, error format |
| `docs/canon/` | the one right way per task |
| `docs/std/` | standard library reference |
| `docs/tools/` | agent tools written in Llrst |
| `docs/workflows/` | step lists for agents |
| `compiler/` | `llrc` (Rust) |
| `std/` | standard library (Llrst) |
| `tools/` | `llr-read llr-grep llr-find llr-edit llr-fmt llr-test llr-diff` (Llrst) |
| `tests/` | golden tests; `tests/run.sh` |

Install: `scripts/install.sh` (see `docs/install.md`). Build only: `cargo build --release`.
