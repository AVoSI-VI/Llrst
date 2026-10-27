# Bootstrap plan (done)

The first plan for Llrst: spec, compiler, std, hello world, tools, install, docs, then
the eleven follow-ups that closed the first review. It was the `plan_todos|11/11` record
in `HANDOFF.md` section 1 until 2026-10-27. The live handoff is
[../HANDOFF.md](../HANDOFF.md). The self-hosting work that followed is in
[self-hosting.md](self-hosting.md).

Status legend: `[ ]` not started, `[~]` in progress, `[x]` done.

## Plan

1. `[x]` Spec: grammar, tokens, types, memory, errors (`docs/spec/`). `6392b37`.
2. `[x]` Compiler `llrc` in Rust on Cranelift: lex, parse, check, MIR, borrowck, codegen (`compiler/src/`). `6392b37`.
3. `[x]` std core, then `os fs path map fmt json cli glob re diff` (`std/`, `docs/std/`). `6392b37`, `0b649a2`.
4. `[x]` Hello world (`examples/hello.llr`). `6392b37`, `53e9f38`.
5. `[x]` Tools `llr-read llr-find llr-grep llr-edit llr-diff llr-test llr-fmt` in Llrst, `std/sexp`, repo formatted with `llr-fmt` (`tools/`). `40a42cd`.
6. `[x]` Install: `scripts/install.sh`; aarch64, riscv64, s390x backends; endian-correct literals. `fde5efb`.
7. `[x]` Docs: tools, workflows, install, README; `HANDOFF.md` for agent handoff. `764e493`, `44368cf`.

## Follow-ups (the eleven)

1. `[x]` `pub` enforcement, strict, per module; `name.private`. `a6f314d`.
2. `[x]` Panic goldens in `tests/panic/`; `idx.oob` carries `len` and `idx`. `9ebe8f0`.
3. `[x]` Flow-sensitive borrow holders; refs stored through `&mut` args are tracked. `0911dd9`.
4. `[x]` `div.zero` is a `P|` line via `sys.checked_div`. `4857a20`.
5. `[x]` `--checked` traps overflow; `wadd wsub wmul wneg` always wrap. `3e45b42`.
6. `[x]` `fs.mtime` and `fs.size` through `runtime/llr_stat.c`. `d5771eb`.
7. `[x]` `W` for unreachable blocks and unused locals (`flow.unreachable`, `name.unused`). `b455b31`.
8. `[x]` Private struct fields unless `(pub (name type))`; `name.private_field`. `e333eee`.
9. `[x]` `vec.sort` and `vec.has` generic over function pointers; stable merge sort. `d65d985`. Replaced by trait bounds in `ec125f7`.
10. `[x]` `map` generic keys via hash and eq; compact tombstones. `2b49824`. Replaced by trait bounds in `ec125f7`.
11. `[x]` `re` captures and word boundaries; `llr-grep --word`, `llr-edit --regex`. `2b3b6a7`.

The status record named only the first eight follow-ups. Items 9 to 11 are the three
remaining 2026-10-15 commits after `ce284b9` and are assumed to be the other three of the
eleven.

Closing status record:

```
commit|ce284b9|tests: lifetimes, struct escape, move through deref
suite|tests/run.sh|done|fails=0
lines|compiler=6018 rs|std=1685 llr|tools=464 llr|max_file=449|budget=500 soft, 1000 hard
plan_todos|11/11 complete
```

## Progress log

- 2026-10-15: `6392b37` llrc bootstrap: spec, compiler, std core, hello world. `0b649a2` std modules and `docs/std`. `40a42cd` seven tools and `std/sexp`. `fde5efb` `install.sh` and the three extra backends. `764e493` docs. `ce284b9` lifetime and escape tests; status record above. `44368cf` `HANDOFF.md`. `53e9f38` `examples/hello.llr`. `87bc5c5` MIT license.
- 2026-10-15: follow-ups 1 to 11 landed the same day (`a6f314d` through `2b3b6a7`).
- 2026-10-18 to 2026-10-19: the second wave (tools `write ls rm git http`, `net thread http gql xml soap grpc mqtt`, traits and `dyn`, `'static` regions, Windows and Darwin hosts) is recorded by the git log, not here; its open limits are the phases in `HANDOFF.md`.
- 2026-10-27: moved here from `HANDOFF.md` section 1.
