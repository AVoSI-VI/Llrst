# langspeed: make Llrst faster, stage 0 to stage 2

Scope, decided 2026-11-08:

- Both kinds of speed: compiler throughput (`llrc`/`llrs` building code, above all
  `self/`) and runtime speed of the code they emit.
- Both compilers are measured: stage 0 (`compiler/`, Rust on Cranelift) and stage 2
  (`llrs2` from `self/`), so the codegen gap between them is a number, not a guess.
- Cross-language benchmarks live in `benchmarks/`. Languages: Llrst, Python, Rust,
  Nim, C, JavaScript, Go, Zig, Java.
- Regressions are caught by a checked-in baseline file, not by eyeballing.
- Llrst code follows `docs/canon/`; the other languages use their standard idiomatic
  form (no tricks that the language's own community would call unrepresentative).

Native x86-64 Linux first (this machine). aarch64, Windows, and wasm are follow-ups.

## Where Llrst is probably slow

Suspects, ordered by expected payoff. Phase 0 measured them on Linux x86-64
(Intel Core Ultra 5 125H). Times are medians from `benchmarks/baseline.txt`.
Stage2/stage0 is the codegen gap. The multiple versus C or Rust is stage 0.

| area | evidence | suspected cost | verdict |
|------|----------|----------------|---------|
| stage 2 codegen has no register allocation and no optimizer | `HANDOFF.md` known gaps; `docs/spec/types.md`: every local is a stack slot, a scalar local lives in a callee-saved register only when one is free | memory traffic on every local read/write in hot loops; no constant folding, CSE, or dead-code removal after MIR | Promoted, and it is the smaller half. On the checked-in run, `fib` is 612/534 ms (1.15x), `sieve` 644/568 (1.13x), `nbody` 715/570 (1.25x). A 200 ms micro median is about 1.0x on `fib` and 1.13x on `sieve`. Stage 0 is already about 20x C on `fib` and 25x C on `sieve`. |
| floats on SysV | `docs/spec/types.md:126`: SysV has no callee-saved XMM, so floats stay in memory | float locals spill on x86-64 even in stage 2's register path | Promoted for `mandelbrot` only. Stage 2 is 203/73 ms (2.8x) there, against 1.15x on integer `fib`. `nbody` (1.25x) and `spectral-norm` (290/215, 1.35x) do not add a float penalty on top of the integer gap. Stage 0 `nbody` is 570 ms against about 10-20 ms for C and Rust, so the bodies are slow before spills. |
| allocator | `std/alloc.llr`: free-list `find` is a first-fit linear scan; every block pays a 16-byte header and 16-byte alignment | allocation-heavy code degrades as the free list grows; small allocations carry header overhead | Promoted as a stage 0 cost. `binary-trees` 199 vs C 19, `string-build` 708 vs Rust 28 (25x), `map-ops` 976 vs Rust 166 (5.9x). Stage 2 is 1.25x, 1.29x, and 0.95x on those runs. Micro `alloc_churn` is 1.31x. |
| `vec.sort` allocates a scratch buffer per call | `HANDOFF.md` perf notes | one allocation plus a full copy per sort | Promoted, not the whole gap. `sort` stage 0 is 445 vs Rust 48 (9.3x) and vs C 157 (2.8x). Stage 2 is 641 (1.44x). Micro `sort` is 1.42x. One scratch buffer cannot explain 9x versus Rust. |
| bounds and check traffic | `std/vec.llr`: `vec.get` and `vec.set` are inline but run `vec.check` per access; div lowers through `sys.checked_div` (`self/lower_div.llr`) | per-access branch in inner loops; division-heavy code pays a call | Not the original 20x. After the trace calls were removed, it is the remaining sieve and nbody gap to Node: every index is still checked, and `aval`'s integer `/` is still a call. |
| string building | `std/str.llr`: `push`/`clone` copy through the allocator; no small-string or reserve-first fast path is visible | repeated realloc and copy when building strings byte by byte | Promoted. Worst cross-language gaps after the parser. `string-build` stage 0 is 708 vs C 45 (16x) and Rust 28 (25x). `json-parse` is 3667 vs C 72 (51x) and Rust 57 (64x). Stage 2 is 1.29x and 1.19x on those runs. Micro `string_build` is 1.29x. |
| drop glue and cloning | `self/cg_drop.llr`, ownership in `self/holders.llr`/`loans.llr`; canon code clones to satisfy the borrow checker | copies and drop calls where a move or borrow would do | Killed as its own row. `json-parse` and `map-ops` are slow, and nothing in this set measures drop glue apart from string building and the map. The extra of `json-parse` over `string-build` matches parser work. |
| compiler throughput | `HANDOFF.md` bench log: `llrs0` builds `self/llrs.llr` in 19.0s, `llrs1/llrs0=1.14` (the self-hosted compiler is ~14% slower than what stage 0 produces) | every `scripts/self.sh` run and every edit-check cycle; front end (`load`, `collect`, `resolve`, `check`, borrow) plus unoptimized stage 2 codegen multiply each other | Phase 4 cut the borrow fixpoint. Phase 5 removed the pthread trace from every call, so the self-build dropped with it: `llrs0` 12.0s to 4.2s, `llrs1` 13.8s to 5.9s. `llrs1/llrs0` is 1.40, up from 1.15, because Cranelift pulls ahead once both compilers stop paying for the trace. |
| bench clock granularity | `std/time.llr`: `time.mono_ns` | micro benches under ~20 ms rounds are noise | Fixed in Phase 0. `time.mono_ns` is one C function on POSIX and Windows, a wasm wrapper, and `std/time.llr`. `llr-bench` times with that clock and stops a round at 200 ms or 1048576 calls. Not a language-speed item. |

## The benchmarks directory

Created in Phase 0:

```
benchmarks/
  README.md          ; how to run, how to read results, fairness rules
  run.sh             ; run.ps1 twin, same flags (repo convention)
  baseline.txt       ; checked in; the regression contract
  micro/             ; Llrst-only (bench ...) files, run by llr-bench
    fib.llr sieve.llr sort.llr string_build.llr map_ops.llr alloc_churn.llr
  tasks/             ; one directory per cross-language task
    nbody/           ; nbody.llr nbody.py nbody.rs nbody.nim nbody.c
                     ; nbody.js nbody.go nbody.zig Main.java expected.txt
    mandelbrot/ spectral_norm/ fannkuch/ binary_trees/
    fib/ sieve/ sort/ string_build/ map_ops/ json_parse/
  compile/           ; compile-time targets and their build commands
    self_build.txt   ; scripts/self.sh --stage, timed per stage
    std_build.txt    ; one large std file through llrc and llrs2
  results/           ; raw run output, gitignored
```

Task set is deliberately mixed: Benchmarks Game classics (nbody, mandelbrot,
spectral-norm, fannkuch, binary-trees) stress float math, loops, and allocation;
Llrst-relevant tasks (fib, sieve, sort, string-build, map-ops, json-parse) stress the
std paths Llrst programs actually hit. Each task takes a fixed size, does the work,
and prints one checksum line. `run.sh` diffs stdout against `expected.txt` before it
believes any timing.

Fairness rules, same for every language:

- Same size, same algorithm, same checksum. Sizes are set so Python finishes in
  roughly 1-10 s and Llrst in well under that.
- Standard build for each language, no external dependencies, no SIMD intrinsics,
  no threads (single-thread round first; a threaded round is a separate task):
  - Llrst: `llrc build` and `llrs2 build`, default flags (no `--checked`)
  - Rust: `rustc -O`
  - Nim: `nim c -d:release`
  - C: `gcc -O2`
  - Python: `python3`, stdlib only, no numpy
  - JavaScript: `node`
  - Go: `go build`
  - Zig: `zig build-exe -O ReleaseFast`
  - Java: `javac` + `java` (JIT warmup is part of the number; note it in the README)
- A language whose toolchain is missing prints `skip|<task>|<lang>|not-installed`
  and is not a failure. Installing a missing toolchain is not a blocker.
- Timing: 5 runs, report min and median. Cross-language process timing happens in
  `run.sh`, never inside the task program.

## Regression detection

`baseline.txt` is checked in and uses the pipe grammar the repo already speaks
(`bench|NAME|ns` from `llr-bench`):

```
bench|stage0|fib|ns
bench|stage2|fib|ns
cross|nbody|llr-stage0|ms
cross|nbody|llr-stage2|ms
cross|nbody|rust|ms
compile|llrs0|self/llrs.llr|ms
```

- `benchmarks/run.sh` runs everything, verifies checksums, prints the table.
- `benchmarks/run.sh --check` additionally fails (exit 1,
  `fail|<name>|base=<n>|now=<n>|+<pct>%`) when any Llrst line is more than 30% slower
  than baseline. Other languages are reported, never gated: their compilers change
  for reasons outside this repo. The first repeats of an unchanged tree on this
  machine moved a median-of-five cross line by up to 26% and a one-sample compile
  by 28%. A 20 ms micro round moved by up to 80%, so `llr-bench` now runs a round
  to 200 ms. 10% was below that noise. 30% still fails a doubled line.
- `benchmarks/run.sh --update-baseline` rewrites `baseline.txt`. It is run only in
  the same commit as the change that moved the number, and the commit says which
  lines moved and why.
- Numbers are per-machine. The baseline names its machine in a header line
  (`# machine|...`); a `--check` on a different machine is advisory only.
- Any commit that touches `self/cg_*.llr`, `self/isa_*.llr`, `self/inline.llr`,
  `compiler/src/codegen/`, `compiler/src/mir/`, or a hot `std/` file must run
  `run.sh --check` before landing. This joins the existing phase-done bar below.

## Phases

Each item is its own commit where the repo already works that way. The global bar
never moves: `tests/run.sh` fails=0 under stage 0 and under stage 2
(`scripts/self.sh`), and `tests/check.sh`/`tests/mir.sh` stay byte-equal.

### Phase 0: measure first

- [x] Add `time.mono_ns` (runtime C, `std/time.llr`, `std/sys_*.llr`, stage 0 and
  `self/` together) or, if that balloons, raise `llr-bench` rounds so ms granularity
  is under 1% error. Decide from one experiment, then stop. Added. The surface is
  one extern per host plus `time.mono_ns`. `llr-bench` divides that clock.
- [x] Create `benchmarks/` as laid out above: all tasks in all nine language slots,
  `expected.txt` per task, `run.sh` (+ `run.ps1`) with `--check` and
  `--update-baseline`, `results/` gitignored.
- [x] First full run; write the first `baseline.txt`; record stage 0 vs stage 2 and
  Llrst vs Rust/C/Nim ratios in `HANDOFF.md` next to the existing `llrs1/llrs0`
  line.
- [x] Promote or kill every row of the suspects table from the numbers. Write the
  verdict back into that table.

Stage 2 misparses long fractional literals (a value like `0.00166...` comes out
about 10x too large; smaller magnitudes miss by more). `nbody` writes those
coefficients as a quotient of a number at least 1 so both compilers print `-1690`.
That is a parser bug, not a speed row.

Exit: `benchmarks/run.sh --check` passes against the fresh baseline, and the
suspects table has no unmeasured rows.

### Phase 1: std and runtime wins (no codegen change)

Guided by `sort`, `string-build`, `map-ops`, `alloc_churn`, `binary-trees`,
`json-parse`. Candidate items, kept only where the bench moves:

- [x] `std/alloc.llr`: size-class the free list or make `find` skip too-small blocks
  faster; keep the 2 GiB cap and header format unless a bench justifies changing
  them. Size-class heads live in one directory block. Native `sys.alloc` stays
  libc, so this does not move the Linux lines; `alloc_churn` moved from the vec
  growth change.
- [x] `vec.sort`: reuse or avoid the scratch allocation (`HANDOFF.md` perf note).
  The scratch buffer is still allocated; the input is no longer copied into it
  before the first merge. A run longer than 32 slots is one `memmove`.
- [x] `vec`: growth policy and `push` inlining; `reserve` before known-size builds
  in `str` and `json` paths. A single push still starts at 8 and doubles. A
  reserve of a known length allocates that many slots. `push` checks capacity
  before calling `reserve`.
- [x] `map`: hash quality and resize policy on the insert/lookup bench. The i64
  hash was already uniform on the bench keys, so it stayed. Heads grow by 4x at
  the same load cap, and `put` hashes once unless a rehash moves the bucket.
- [x] `str`: `push_byte` growth batching; remove clone-then-copy pairs the canon
  files (`docs/canon/strings.md`, `collections.md`) already mark as non-canon.
  One byte grows by 32. `from_int` writes digits into the result. `from` and
  `cstr` copy once. Plain JSON strings are one slice.

Exit: each landed item shows its bench line improving in the same commit's baseline
update; no micro bench regresses past the regression gate.

### Phase 2: stage 2 codegen

The big one: `self/` has no register allocation and no optimizer
(`HANDOFF.md` known gaps). Work in the order the stage0/stage2 gap benches dictate:

- [x] Extend the scalar-in-register rule (`docs/spec/types.md:126`): more locals in
  registers, floats included on SysV (caller-saved XMM around calls is allowed to
  be phase 2a if callee-saved save/restore is the blocker).
  SysV floats live in xmm8-15 as caller-saved registers: `cg_frame` spills only the
  FPR locals that `liveness` says are live across each call (`cg_live.keep_live`),
  and the prologue keeps only float params. Reg-reg float moves are `movaps`; a
  `movsd` merge serialized every op through xmm0 (`mandelbrot` 2.8x to 1.05x).
  Register pressure drops because `optfwd` removes the temps that used to take
  the five SysV GPRs: a local with no use left gets no register (`cg_slot.uses`).
- [x] Kill redundant load/store pairs to stack slots in `self/cg_*.llr` output
  (a small peephole over emitted code, or track the slot's current register in
  `cg_slot.llr`).
  A plain memory scalar is read and written at `fp+disp` (`cg_op.mem_of`,
  `load_at`, `store_at`) instead of `lea; mov; mov`; a binop's right operand goes
  straight into `tmp`/`fp1` when it is a constant or a plain local, and an i32
  constant on an unchecked add/sub or a compare is an immediate; a compare that
  only feeds the block's `if` is one `cmp` and a `jcc` (`cg_live.fuse_ok`); call
  arguments that are 8-byte locals are passed from their slot or register, and a
  scalar result lands in its slot; `isa_x64.copy` uses displacements; `x64_int`
  drops `mov a, b` right after `mov b, a`.
- [x] Constant folding and dead-slot elimination at MIR level in `self/mir.llr` /
  `self/inline.llr`, mirrored in stage 0 (Phase 3) so `emit-mir` stays byte-equal.
  Done as `self/optfwd.llr`, run last by `optmir.run`: a single-assignment
  `_a = copy|move _b`, `_a = & (*_b)`, or `_a = <const>` is forwarded into every
  read and removed. It runs after `emit-mir`, like the rest of `optmir`, so stage 0
  needs no mirror and `tests/mir.sh` stays byte-equal. Arithmetic folding is not
  needed by the gap benches and is left to Phase 3.
- [ ] RISC-V (`self/isa_rv*.llr`) and wasm (`self/wasm_*.llr`) get the same treatment
  only after x86-64/aarch64 numbers land; they are not on the critical path.
  `optfwd` and every `cg_op`/`cg_term` change go through `cg_isa` and `optmir`, so
  RISC-V and wasm already get them (`tests/wasm.sh` fails=0 under a `node:wasi`
  stand-in for wasmtime). Still open: the displacement form of
  `isa_a64_frame.copy_off`, and aarch64 numbers, which need the Mac.

Exit: stage 2 within 10% of stage 0 on `fib`, `sieve`, `nbody`, `spectral-norm`;
baseline updated; the `llrs1/llrs0` self-build ratio in `HANDOFF.md` re-measured.
Met 2026-11-09 on Linux x86-64 (median of five, `benchmarks/baseline.txt`): `fib`
1.05x, `sieve` 1.05x, `nbody` 1.08x, `spectral_norm` 1.03x; every cross task is at
most 1.08x (`binary_trees` 1.31x to 0.98x, `mandelbrot` 1.74x to 1.05x). The
self-build ratio `llrs1/llrs0` is 1.15 (16.8s/14.6s) against 1.22 for the previous
commit on the same machine; `llrs0` itself is 6% slower on that build because
codegen now runs liveness and `optfwd` per body.

### Phase 3: stage 0 and shared MIR

Stage 0 already runs Cranelift at `opt_level=speed`, so the wins here are in MIR and
in what both compilers share:

- [x] Inline policy: measure `self/inline.llr` / `compiler/src/inl.rs` thresholds
  against `fib`, `map-ops`, `json-parse`; tune or leave, with numbers.
  Left at 0 (explicit `(inline ...)` only). A limit of 12 inlines `vec.len`,
  and `vec.sort` then fails `borrow.mut_shared|ref=n` because the result keeps
  `self`'s loan. `Eq.eq` on `i64` is 5 expression nodes and `Hash.hash` on
  `i64` is 8; `fib` has no such callee. Inline frames are pushed only around
  calls, and only when the inlined region is not a loop, so `vec.get` and
  `str.at` do not call `llr_trace_push` on the success path. Panic traces stay
  `vec.check`, `vec.get`, `main.main`.
- [x] Drop-glue and clone elision where `holders`/`loans` already prove a move:
  fewer copies on `json-parse` and `map-ops`. A `std` `clone` or `from` whose
  argument is a loan of a local with no later use and no live holder becomes a
  move. A whole-local drop whose flag is definitely clear is removed. One
  sample: stage 0 `map-ops` 796 ms to 346 ms, `json-parse` 2633 ms to 1829 ms.
- [x] The passes run after `emit-mir` and after borrowck, in stage 0 and
  `self/` (native and wasm). `tests/mir.sh` and `tests/check.sh` stay
  byte-equal. `tests/run.sh` is fails=0 under stage 0, and stage 2 built from
  this tree matches stage 0 on check and emit-mir.

Exit: stage 0 numbers improve or hold on every micro bench; check/mir goldens
byte-equal; no cross-language task regresses. Median of five micro runs, stage
0 ns/call, baseline to now: `fib` 54436 to 42086, `sieve` 151098 to 74822,
`sort` 48891 to 35529, `string_build` 12582 to 9522, `map_ops` 20608 to 15669,
`alloc_churn` 11780 to 9027. One cross sample improved every Llrst line; `fib`
stage 2 (524 ms to 485 ms) and `string-build` stayed inside the 26% noise band
and were left in `baseline.txt`.

### Phase 4: compiler throughput

- [x] Add per-phase timing to the driver (`self/drive.llr`, stage 0 `driver.rs`):
  load, parse, collect, resolve, check, borrow, lower, codegen, link. Print behind
  a flag; keep default output unchanged.
  `LLRST_TIME` (any value, unset by default) prints `time|<phase>|<ms>` for those
  nine phases on stderr. Stage 0 is `compiler/src/phase.rs`; stage 2 is
  `self/phase.llr`. Object emission is part of codegen.
- [x] Attack the largest phase only. Likely candidates: repeated `last_seg`-style
  string scans (`HANDOFF.md` std-gap table lists several duplicated helpers),
  re-parsing `std` per build, codegen of `self/` once Phase 2 makes stage 2 code
  itself faster (the 1.14 ratio compounds here).
  On `llrs0` building `self/llrs.llr`, borrow was 9.0s of 14.0s. Inside it,
  holder fixpoint plus loan-liveness was 6.1s and move checking was 1.9s. Parse
  was 0.4s, so re-parsing `std` was not the phase to attack. `holders.or_span`
  and `kill_local` now copy or clear one local's loan bits a machine word at a
  time instead of one `borrow.bit`/`borrow.setb` call per loan.
- [x] Re-run `compile` benches: `self/llrs.llr` build per stage, one large `std`
  file, `tests/run.sh` wall time.
  One sample, `benchmarks/run.sh --once --only compile`, same machine as
  `baseline.txt`: llrc 3127 ms (was 3117), llrs0 12040 (was 14631), llrs1 13814
  (was 16813), http check llrc 32 ms, llrs2 180 (was 199). `scripts/self.sh
  --stage` wall time was 29.0s. `tests/run.sh` under that `llrs2` is fails=0;
  its wall time was not recorded as a bench line.

Exit: `llrs1/llrs0` is 1.15 (13.8s/12.0s), not 1.05. The reason is in
`HANDOFF.md`: both compilers run the same `self/` algorithm, phase 2 already
put integer loops within 5% of stage 0, and this cut moved both stages by the
same fraction. What is left is stage 2 versus Cranelift on the borrow checker,
still the largest phase (7.4s of the 12.0s `llrs0` build). `scripts/self.sh
--stage` is 29.0s against the Phase 0 total of 51.8s (llrc 4.5s + llrs0 21.3s
+ llrs1 26.0s). `self|stage2=equal`. `tests/check.sh` and `tests/mir.sh` fails=0.
`tests/run.sh` under `llrs2` is fails=0.

### Phase 5: close the cross-language gap, then lock it in

- [x] Full `benchmarks/run.sh` on the finished tree. Remaining gaps to Rust/C/Nim
  are listed by task with the known cause or `unknown` next to each; `unknown` is
  an acceptable final state, an unlisted gap is not.
- [x] Update `baseline.txt`, the suspects table verdicts, and `HANDOFF.md` (perf
  notes and bench log).
- [x] Note in `benchmarks/README.md` how a future perf commit runs `--check`, so
  the harness survives this plan.

What landed, x86-64 SysV only. The prologue no longer calls `llr_trace_push` /
`llr_trace_pop` (`pthread_once` plus two TLS calls on every function entry).
Stage 0 records each function's address span and the walker names the caller
from the return address. Stage 2 stores `0x4C4C5254` at `fp-16` and the name at
`fp-8`; inline frames still push on the panic path. `vec.set` is `(inline ...)`,
same as `vec.get`. JSON `true` and `false` are slices of the source, same as
plain strings and numbers. Other targets keep the pthread (or Windows TLS) calls.

Median of five, stage 2, milliseconds, from `benchmarks/baseline.txt`
(2026-11-09). A gap over 2x to Rust, C, or Nim is one line. `unknown` is not
needed.

| task | stage 2 | node | python | C | Rust | Nim | cause where the gap is over 2x, or where Node or Python wins |
|------|---------|------|--------|---|------|-----|--------------------------------------------------------------|
| fib | 133 | 138 | 1163 | 21 | 32 | 22 | C, Rust, and Nim rewrite the pure recursion. Llrst emits one call per frame. |
| sieve | 90 | 49 | 499 | 20 | 19 | 20 | Every mark is a bounds-checked byte store. The check is not removed. |
| sort | 199 | 433 | 366 | 105 | 27 | 157 | Rust's sort is specialized. Llrst is a stable mergesort with a scratch buffer (1.9x C, 1.3x Nim). |
| string_build | 113 | 139 | 367 | 36 | 26 | 39 | Each append grows an owned `str`. There is no small-string path. |
| map_ops | 157 | 296 | 362 | 44 | 66 | 125 | Chained map, one hash per put (1.3x Nim). C and Rust use a tighter table. |
| json_parse | 896 | 307 | 379 | 54 | 42 | 522 | The task builds the text with `str.push` and parses an owned DOM, one vec per object. Node and Python call a native parser. Nim's std parser is 1.7x. |
| nbody | 113 | 38 | 310 | 13 | 12 | 14 | `at` and `put` stay calls because they call `vec.get` / `vec.set`, and each index is checked. |
| mandelbrot | 61 | 41 | 634 | 22 | 25 | 22 | `pixel` is one call per point. |
| spectral_norm | 53 | 35 | 585 | 7 | 7 | 13 | `aval`'s integer `/` lowers to `sys.checked_div`, a call in the inner loop. |
| fannkuch | 81 | 33 | 210 | 11 | 11 | 11 | `flips` clones the permutation on every call, and the swaps are bounds-checked. |
| binary_trees | 52 | 39 | 194 | 17 | 23 | 20 | Each node is a two-slot vec allocation, not a raw pair of pointers. |

Stage 2 beats Python on every task except `json_parse` (896 vs 379). It beats
Node on `fib`, `sort`, `string_build`, and `map_ops`. It does not beat Node on
`sieve`, `json_parse`, `nbody`, `mandelbrot`, `spectral_norm`, `fannkuch`, or
`binary_trees`. Raising the free-function inline limit inlined `aval` and made
stage 0 `spectral_norm` slower, so the limit stayed 0. `tests/run.sh` fails=0
under stage 0 and under stage 2. `scripts/self.sh --stage` is `stage2=equal`.
`tests/check.sh` and `tests/mir.sh` fails=0.

Exit: not met. The Node and Python bar above is the part that is still open.
Every Rust, C, and Nim gap is under 2x or named in the table.

## Non-goals

- No new language features to make benchmarks faster (no SIMD, no threads, no
  `@inline` hints beyond the existing `(inline ...)` form) until Phase 5 is done
  and a gap is proven to need one.
- No benchmark-specific special cases in the compiler or std. If a change only
  helps one task, it does not land.
- No CI runner work: the baseline gate is a local, pre-commit check on one named
  machine, matching how `tests/run.sh` is already the bar.
