# Self-hosting roadmap (done)

This was section 10 of `HANDOFF.md` until 2026-10-27. Every step is closed. It is kept
so the next agent can see what was planned, what closed it, and in which commit. The
live handoff is [../HANDOFF.md](../HANDOFF.md). The text below is kept as written; its
"section N" references are to the `HANDOFF.md` of that time: 1 status, 3 repo map,
4 pipeline, 5 build and verify, 6 implementation notes, 8 next steps, 9 conventions.

Status legend: `[ ]` not started, `[~]` in progress, `[x]` done.

Goal: a compiler for Llrst written in Llrst that emits native x86-64 objects, builds
itself, and passes `tests/run.sh` and `tests/run.ps1`. `llrc` (Rust, Cranelift) stays the
reference compiler until stage 2 reproduces itself byte for byte; every step below is
verified by comparing against `llrc` output, never by inspection.

Starting point: the reader (`std/sexp.llr`), the parser (`std/ast.llr`, `std/ast_expr.llr`)
and a typed checker (`std/check.llr`, `std/check_ty.llr`, `std/check_expr.llr`, see
`docs/tools/llr-check.md`) exist. `tests/run.sh` compares `llr-check` with
`llrc check` on every `tests/err` golden whose code is not `borrow.*`, `move.*`, or `init.*`.
`llrs emit-mir` prints a full MIR dump, byte-equal to `llrc emit-mir` on every `tests/run/*.llr` and `std/*.llr` (`tests/mir.sh`, `tests/mir.ps1`). `llrs check` stderr is byte-equal to `llrc check` on every `tests/err` and `tests/warn` file, including `--deny` (`tests/check.sh`, `tests/check.ps1`).

Layout: a new flat `self/` directory of `proj.` modules (`proj.X` loads `X.llr` beside the
main file, flat only), entry `self/llrs.llr`. The name `llrs` is a placeholder. Files under
500 lines and functions under 40 lines, as section 9. The system C compiler leaves the
codegen path and remains only as the linker driver, as `link.rs` uses it now. The
`runtime/*.c` shims stay C and are prebuilt by install.

## Steps

1. `[x]` x86-64 encoder. `self/x64.llr`: the integer, SSE2, and control-flow subset the MIR
   needs, labels and fixups, SysV and Win64 calling conventions. Verify: `(test ...)`
   forms with expected byte strings per instruction; `llr-test self/x64.llr` passes.
   Closed by `eca4078` x86-64 encoder (self/x64.llr).
2. `[x]` Object writers. `self/elf.llr` (ELF64 relocatable: text, data, rodata, bss, symtab,
   strtab, PC32, PLT32, and 64 relocations), later `self/coff.llr` (x86-64 COFF, REL32 and
   ADDR64). Verify: a hand-built object exporting `main` that calls `write` links with the
   same `cc` command `link.rs` runs and prints one record.
   Closed by `5d7afeb` ELF64 object writer; COFF by `0392b3e` Windows path: a COFF writer.
3. `[x]` Codegen. `self/cg*.llr` reproducing the ABI in section 6: scalars by value, aggregates
   by pointer copied in the prologue, sret, every local a stack slot, drop glue, `llr_trace`
   panic frames, 24-byte static `str` records, `main` storing `llr_argc` and `llr_argv`,
   the `--test` entry, `--checked` traps, shift masking, saturating float-to-int unless `--checked`. The CLI
   is identical to `llrc` (`build run check emit-mir`, `-o --std --test --checked --deny`).
   Verify: `LLRC=self/llrs tests/run.sh` reports `done|fails=0` on Linux.
   Closed by `e04e8e9` codegen.
4. `[x]` Stage bootstrap. `scripts/self.sh`: stage 0 `llrc` builds `llrs0`; `llrs0` builds
   `llrs1`; `llrs1` builds `llrs2`; `cmp llrs1 llrs2` is byte-equal (deterministic symbol
   order, no timestamps); then `LLRC=llrs2 tests/run.sh`. Add a `self|stage2=equal` record
   to the section 1 status block once it holds.
   Closed by `40dcadd` bootstrap. The status record was added to `HANDOFF.md` on 2026-10-27.
5. `[x]` Windows. The COFF writer, the Win64 ABI, `cl ... /link /STACK:8388608`, and the
    prebuilt `llr_win.obj` and `llr_wsa.obj` (install compiles them once). Verify:
    `tests/run.ps1` with `$env:LLRC` pointing at `llrs2` ends `done|fails=0`.
    Closed by `0392b3e` Windows path: a COFF writer.
6. `[x]` Retire. `install.sh` and `install.ps1` build `llrc` only as stage 0 and install `llrs2`;
    update sections 3, 4, and 5 and `docs/install.md`; move anything `llrs` cannot do to
    section 8. Keep `llrc` in the tree until two releases agree.
    Closed by `54ed8fe` Retire: install llrs2. The section 3, 4, 5 updates were partial;
    the rest is Phase 0 of `HANDOFF.md`.

Notes. A language gap found while writing `self/` is a section 8 item first (reuse, then
refactor, then new code); do not widen the language only for the compiler. Expect roughly
the Rust line count, about 6000 lines in 15 to 20 files. `std/map.llr` iterates in
insertion order and `std/fs.llr` writes raw bytes, so deterministic object output needs
no new std. Outcome: `self/` is 56 files and 11972 lines, about twice the estimate.

## Progress log

- 2026-10-18: `0e21147` ast: parse sexp trees into the surface grammar. `25c7d6b` check: match llrc on type and name errors. The starting point above.
- 2026-10-19: `c094862` add roadmap to self hosting Llrst. Plan written as section 10.
- 2026-10-20: `5ba7ad2` emit the five missing type codes from llr-check. `d684180` differential check harness (`tests/self.sh`, `tests/self.ps1`).
- 2026-10-22: `ea92162` finish 1 in section 10. `5691d5a` typed tree checker (`self/thir*.llr`). `921568a` instance worklist (`self/mono.llr`, `self/subst.llr`; later superseded by `self/drive.llr`).
- 2026-10-23: `5bc7568` MIR that matches llrc emit-mir. `ef8ddea` finish the self-hosted MIR dump (`tests/mir.sh`, `tests/mir.ps1`). `41b70a5` self-hosted borrow check.
- 2026-10-26: `eca4078` step 1 encoder. `5d7afeb` step 2 ELF64. `e04e8e9` step 3 codegen. `40dcadd` step 4 bootstrap, `self|stage2=equal`. `0392b3e` step 5 COFF writer and Windows. `54ed8fe` step 6 retire, `llrs2` installed as `llrc`. Merged as `ebaed76`.
- 2026-10-27: moved here from `HANDOFF.md` section 10.
