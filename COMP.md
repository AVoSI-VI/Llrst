# Llrst: compile-speed plan for `self/`

This file is the work plan for making the self-hosted compiler (`self/`, binary `llrs`,
installed as `llrc`) compile faster. It is written so another agent can pick it up cold.
Read it, then [OPT.md](OPT.md) (the findings this plan schedules; items are cited by
number as "OPT 1" .. "OPT 15"), then the **Rules for this work** in
[HANDOFF.md](HANDOFF.md). When code and this file disagree, trust the code and fix this
file. OPT.md is the evidence record and is not edited except to append measurements.

Status legend: `[ ]` not started, `[~]` in progress, `[x]` done. Update the boxes as you
go and add a dated line to the **Progress log** at the bottom.

## The problem in one paragraph

`llrs` compiles `self/llrs.llr` (92 files, 17,031 lines, ~3,600 fn defs with `std/`) in
355 s; stage 0 (`compiler/`, Rust on Cranelift) compiles the same input in 6.7 s. `check`
alone is 244 s. Nothing in the backend explains that: `llrs1` (compiled by `llrs`, no
register allocator) is only 10% slower than `llrs0` (compiled by Cranelift) on the same
work (390 s vs 355 s, OPT **Measurements**). The gap is algorithmic: linear symbol
scans, a quadratic unused-fn lint, every user fn checked twice, inline callees
re-typechecked per call site, and per-pass allocation in dataflow. This plan removes
those in order of expected payoff while keeping every output byte identical.

## Rules for this work

These add to the rules in `HANDOFF.md` (500-line files, 40-line fns, `llr-fmt`, one
commit per phase, suite green at the end of every phase).

- **Byte-equal output is the invariant.** No phase may change what `llrs` writes:
  `check` stderr, `emit-mir` stdout, object bytes, symbol names, executables. The
  checks are listed under **Verification**; all of them pass at the end of every phase.
  If a change cannot be made output-neutral, it does not belong in this plan.
- **Do not change `std/`, the language, or stage 0 for speed.** `std/` may gain a
  function only if the same function is needed by code outside `self/`; a change there
  needs a `(test ...)`, a `docs/std` row, and `tests/mir.sh` still byte-equal. No
  change goes to `compiler/`.
- **Measure before and after every phase** with the commands under **Measuring**, using
  `llrs0` (built by stage 0) so numbers reflect the algorithm and not the backend.
  Record both numbers in the phase's row in **Results** and in the **Progress log**.
  A phase that does not move the number it targets is reverted, not kept.
- **No caches that outlive the process** (no files written beside the exe, no env-var
  side effects) unless the phase says so explicitly (Phase 7 discusses one and rejects
  it).
- `prog.prog` is `(& prog.prog)` everywhere after `collect`. Every lookup table added
  to `prog` is built once at the end of collection and read-only afterwards. Memoizing
  into `prog` during checking or lowering is not possible without `&mut`; do not try.
- The std hash map is `std/map.llr`: `(map K V)` with `new put get get_mut has del len`.
  `Hash` and `Eq` exist for `i64 str u8 i32 u32 u64 bool` (`std/cmp.llr`). `get` with a
  `str` key accepts a literal or `(& str)`. `self/` does not use it yet; this plan
  introduces it.

## Baseline

From OPT **Measurements** (commit `7077aa6`, Windows 11, 8 logical CPUs, MSVC `cl`,
`SOURCE_DATE_EPOCH=0`, one run each). Re-measure in Phase 0 and replace this table.

| compiler | command | input | wall time |
| --- | --- | --- | --- |
| stage 0 | `check` | `self/llrs.llr` | 0.69 s |
| stage 0 | `build` | `self/llrs.llr` | 6.66 s |
| `llrs0` | `check` | `self/llrs.llr` | 244.38 s |
| `llrs0` | `build --emit-obj` | `self/llrs.llr` | 344.42 s |
| `llrs0` | `build` | `self/llrs.llr` | 355.36 s |
| `llrs1` | `build` | `self/llrs.llr` | 390.12 s |
| `llrs0` | `check` | `examples/hello.llr` | 1.62 s |
| `llrs0` | `build` | `examples/hello.llr` | 5.96 s |

Target at the end of Phase 7: `llrs0 build self/llrs.llr` under 60 s, `check` under
20 s, `hello` build under 2.5 s. These are goals, not promises; the phase tables say
what each step is expected to remove.

## Results

Fill one row per phase as it lands. `check` and `build` are `llrs0` on `self/llrs.llr`
in seconds; `hello` is `llrs0 build examples/hello.llr`.

| phase | commit | check | build --emit-obj | build | hello build |
| --- | --- | --- | --- | --- | --- |
| baseline (OPT) | `7077aa6` | 244.4 | 344.4 | 355.4 | 5.96 |
| 0 | | | | | |
| 1 | | | | | |
| 2 | | | | | |
| 3 | | | | | |
| 4 | | | | | |
| 5 | | | | | |
| 6 | | | | | |
| 7 | | | | | |

## Verification

Run all of these at the end of every phase. Windows is the measurement host; the `.sh`
twins exist for Linux and Darwin.

```
cargo build --release                                      # stage 0; once per checkout, and after any pull that touches compiler/
powershell -File scripts\self.ps1 --stage                  # llrs0, llrs1, llrs2 under target\self; must print self|stage2=equal
powershell -File tests\check.ps1 target\self\llrs0.exe     # check stderr byte-equal to stage 0 on tests/err and tests/warn
powershell -File tests\mir.ps1 target\self\llrs0.exe       # emit-mir byte-equal to stage 0 on tests/run and std
powershell -File tests\x64_ref.ps1                         # every object byte-equal to the Phase 0 snapshot (Phase 0 adds this script)
$env:LLRC = "$PWD\target\self\llrs2.exe"; powershell -File tests\run.ps1   # goldens, std tests, ui, line check
```

What each one proves:

- `stage2=equal` proves the new compiler is a fixpoint of itself. It does **not** prove
  the output equals the old compiler's output; `x64_ref.ps1` does that.
- `check.ps1` and `mir.ps1` prove the frontend (typecheck, lower, borrowck, lint) still
  produces the same diagnostics and the same MIR as stage 0. Every phase that touches
  `thir*`, `lower*`, `borrow`, `holders`, `liveness`, `moves`, `lint`, `inline`, or
  `prog` relies on these.
- `x64_ref.ps1` compares every `--emit-obj` object (`tests/run/*.llr` with a `main`,
  `std/*.llr --test`, and `self/llrs.llr`, each as ELF and COFF) against a snapshot
  taken in Phase 0 from the **baseline** compiler. This is the only check that catches a
  codegen-side change (Phase 6, Phase 7) that still self-bootstraps. It takes about 20
  minutes before Phase 1 and gets faster with every phase.
- `run.ps1` under `llrs2` proves the compiled programs still behave.

The stale binary trap from OPT: a committed `target/release/llrc.exe` older than
`std/set.llr` fails to parse `std`. Always `cargo build --release` first.

## Measuring

```
cargo build --release
$env:LLRST_STD = "$PWD\std"; $env:LLRST_RUNTIME = "$PWD\runtime"; $env:SOURCE_DATE_EPOCH = '0'
powershell -File scripts\self.ps1 --stage            # produces target\self\llrs0.exe from the current self/
$l = "$PWD\target\self\llrs0.exe"
$env:LLRST_TIME = '1'                                # Phase 0 adds this; prints time|<phase>|<ms> on stderr
Measure-Command { & $l check self\llrs.llr --std std }
Measure-Command { & $l build self\llrs.llr -o target\self\work\llrs_t --std std --emit-obj }
Measure-Command { & $l build self\llrs.llr -o target\self\work\llrs_t --std std }
Measure-Command { & $l build examples\hello.llr -o target\self\work\hello --std std }
Remove-Item Env:LLRST_TIME
```

Use `llrs0`, not `llrs1` or the installed `llrc`: `llrs0` is compiled by Cranelift, so a
change in its wall time is a change in the algorithm, not in `self/`'s own codegen. Run
each command twice and keep the second number; expect a few percent of noise. Keep the
baseline binary (`target\self\base\llrs0.exe`, saved in Phase 0) to re-measure the old
numbers on the same machine state when a result looks odd.

There is no sampling profiler for `llrs` output (objects carry no debug info and every fn
symbol is `local`), so the `LLRST_TIME` phase timers from Phase 0 are the profiler.
Splitting a phase further means adding a timer around the suspect loop for the duration
of the investigation and removing it before commit.

## Phases

Order is by expected payoff divided by risk, with the measurement after each phase
deciding whether the next one is still worth doing. Each phase is one commit and lists:
the OPT items it covers, the files it touches, the design, the equivalence argument (why
the output cannot change), the steps, and the commit message.

### Phase 0: harness (no output change) `[ ]`

Covers: nothing in OPT; makes the rest measurable and checkable on Windows.

Files: `self/drive.llr`, `self/llrs.llr`, new `tests/x64_ref.ps1`, this file.

1. `[ ]` `cargo build --release`, then `scripts\self.ps1 --stage`. Copy
   `target\self\llrs0.exe` to `target\self\base\llrs0.exe`. This is the baseline
   compiler for the snapshot and for re-measuring; `target/` is gitignored.
2. `[ ]` `tests/x64_ref.ps1`, a twin of `tests/x64_ref.sh`: `[--save DIR]`; compiler from
   `$env:LLRC` else `target\self\llrs0.exe` (no rebuild logic; the `.sh` calls
   `self.sh --llrs0`, which `self.ps1` does not have, and adding that is not needed).
   Same inputs as the `.sh`: `tests/run/*.llr` containing `^(fn main`, `std/*.llr`
   containing `^(test `, with `--test`, and `self/llrs.llr`; each as
   `x86_64-unknown-linux-gnu` (`.o`) and `x86_64-pc-windows-gnu` (`.obj`); set
   `SOURCE_DATE_EPOCH=0`. Compare with `[System.IO.File]::ReadAllBytes`; print
   `ok|x64/<kind>/<name>` or `fail|x64/<kind>/<name>|first=<offset>` and `done|fails=N`.
   Then `$env:LLRC = target\self\base\llrs0.exe; tests\x64_ref.ps1 --save target\self\x64_ref`.
3. `[ ]` `LLRST_TIME`: in `drive.build`, when `(os.env "LLRST_TIME")` is `opt.some`,
   print `time|check|<ms>`, `time|codegen|<ms>` (the `pump` plus `lint.generics`),
   `time|object|<ms>` (`cg_mod.finish` and `write_obj`), `time|link|<ms>` on stderr
   with `time.mono_ms` (`std/time.llr`). `drive.run` adds `time|load|<ms>` around
   `load.load_at`. `llrs.emit_ok` adds `time|check|<ms>`. Nothing is printed when the
   variable is unset, so no golden changes; `tests/*.ps1` never set it. Keep the timer
   helper to one fn so `build` stays under 40 lines.
4. `[ ]` Measure per **Measuring**; replace the **Baseline** table with the new numbers
   including the `time|` split, and fill the Phase 0 row of **Results**.
5. `[ ]` **Verification** (the snapshot compare is trivially equal here; run it once so
   the script is known to work).
6. `[ ]` Commit: `self: LLRST_TIME phase timers, x64_ref.ps1 snapshot compare`.

### Phase 1: single-pass `lint.items` (OPT 2) `[ ]`

Expected: the largest single cut to `check`. `fn_item` calls `called` for every private
user fn (~1,600 in `self/`), and `called` walks every def body in the program (all 3,600
defs including `std`) calling `prog.lookup_fn` on every name and call node until it finds
a reference. That is on the order of 10^8 `lookup_fn` calls, each a dotted-name split
with two heap `str.slice`s plus linear scans. A single pass over all bodies does one
`lookup_fn` per node, roughly 10^5 calls.

Files: `self/lint.llr` (288 lines; room).

Design. Replace `called`/`refs`/`refs_e`/`test_refs` (lines 164-185) and
`use_seen`/`test_mentions`/`mentions`/`mentions_e`/`has_alias` (120-146) with one walk:

```
(struct marks (fns (vec bool)) (uses (vec (vec bool))))   ; fns by def id; uses[mi][k] for alias k of module mi
```

`items` builds `marks` by walking every def body with that def's module and, when `test`,
every `tdecl` body with its `mod`. At each node of kind `1u8` or `23u8`:

- `(let r (prog.lookup_fn p mi (& (. e text))))`; if `r >= 0` set `fns[r]`.
- for each alias `k` of module `mi`: if `text` starts with `alias` followed by byte `46u8`
  set `uses[mi][k]`. Compare in place (`str.len`, `str.at`, byte loop or `str.starts`
  on the alias then a `str.at` check at `(str.len alias)`); do not build the `pre`
  string `has_alias` builds today.

Then `fn_item` reads `fns[i]` instead of calling `called`, and `mod_items` reads
`uses[mi][k]` instead of `use_seen`.

Equivalence. Today `called p id test` is true iff some def `d` (any module) has a
node whose `lookup_fn p d.module text == id`, or (when `test`) some test `t` has a node
whose `lookup_fn p t.mod text == id`. `fns[id]` is set under exactly that condition.
`use_seen p mi alias test` is true iff some def of module `mi` (or test with `mod == mi`)
has a node whose text starts with `alias.`; `uses[mi][k]` is set under exactly that
condition. `lookup_fn` is pure (`prog` is `&`). Warnings are pushed in the same
`fn_item`/`mod_items` order as before, and `diag.flush` sorts anyway. Nodes are the
same because the walk visits `kids` recursively exactly as `refs_e` and `mentions_e` do.

Traps:

- `fn_item` and `mod_items` skip std modules, but the **walk** must cover std defs too:
  `called` walks every def today, and walking everything is what makes the equivalence
  argument exact. (A std body cannot name a `proj.*` fn, so this costs one pass and
  changes nothing; do not optimize it away.)
- Keep `(and (str.eq (& (. d name)) "main") (== (. d module) (. p main)))` and the
  `is_pub`/`extern` early returns in `fn_item` as they are.
- `lookup_fn` for a kind-1 node that is a local variable name fails through `mod_fn`
  (one linear scan). That stays until Phase 4 replaces the scan with a map.

Steps:

1. `[ ]` Implement `marks`, the walk, and the two reads; delete the replaced fns.
2. `[ ]` `tests\check.ps1 target\self\llrs0.exe` (the `tests/warn` goldens are the
   direct test: `fn.unused`, `use.unused` with and without `--test`).
3. `[ ]` Full **Verification**; measure; fill the row.
4. `[ ]` Commit: `self: lint.items marks references in one pass`.

### Phase 2: reuse `check.run`'s bodies in codegen (OPT 1) `[ ]`

Expected: removes the second typecheck, lower, and borrowck of every non-generic user fn
in `build` (~2,500 fns in `self/`). Bounded above by the 100 s difference between
`check` and `build --emit-obj`, which also contains std fns, generic instances, and
x86-64 emission; expect 30 to 80 s off `build`.

Files: `self/check.llr` (46 lines), `self/drive.llr` (270), `self/wasm_fn.llr` (373),
`self/llrs.llr` (162), `self/mir.llr` (an empty-body constructor; 227 lines).

Design. `check.run` gains a `keep bool` parameter and returns

```
(pub (struct kept (pub (at (vec i64))) (pub (bodies (vec mir.body)))))   ; at[def id] = index into bodies, or -1
```

`lower_one` pushes `b` into `kept` after `borrow.run` and `lint.body` when `keep` is
true (instead of dropping it). `drive.build` calls `check.run ... true`, and `inst`
(`self/drive.llr` 230-241) becomes:

```
if (vec.len args) == 0 and kept.at[id] >= 0:
    b = (mem.replace (vec.get_mut bodies k) (mir.empty_body))   ; take ownership, free as we go
    sym = func_sym ...; cg_term.compile g p (& b) sym true
else: today's path
```

`kept` is threaded from `build` into `pump` -> `drain` -> `inst` as `(&mut check.kept)`.
`wasm_fn.build` and `wasm_fn.inst` (lines 25-36, 67-78) get the same parameter and the
same branch. `llrs.emit_ok` passes `false` and ignores the result.

Equivalence. `thir.check_fn p id args ds` and `lower.lower_fn p e name ds` depend only
on `p` (immutable), `id`, `args`, and `name`; `inline.try_inline` runs inside
`lower_fn` with its own `sink`. `check.run` calls them with `args` empty and `name =
prog.fn_path p id`; `inst` calls them with the same values for a user fn (a user fn has
no `gens`, so its `args` are always empty). The MIR is therefore identical, and
`cg_term.compile` sees the same `(& mir.body)`. Diagnostics: `build` returns before
`pump` if `check.run` left any error; `borrow.run` emits only errors, so the second
`borrow.run` today adds nothing and skipping it changes nothing; `lint.body` already runs
only once (in `check.run` for user fns, in `inst` for the rest). Order of codegen is
unchanged because `inst` is still driven by `pend`.

Traps:

- `check.user_fn` is the predicate for both the `check.run` loop and the `inst` skip of
  `lint.body`; use it for the `kept` branch too, so the three agree.
- Under `--test`, `add_test` lowers test bodies itself; untouched.
- `cmd == "check"` and `emit-mir` keep nothing (`stopped` returns before `pump`).
- Memory: bodies for unreachable user fns stay in `kept` until `build` returns; fine.

Steps:

1. `[ ]` `mir.empty_body`, `check.kept`, `check.run keep`, thread into `drive` and
   `wasm_fn`.
2. `[ ]` `tests\x64_ref.ps1` (objects equal), then full **Verification**; measure; fill
   the row.
3. `[ ]` Commit: `self: codegen reuses the MIR check.run already built`.

### Phase 3: inline callee memo, trace slot counter, late arg clone (OPT 3) `[ ]`

Expected: the four `inline` fns (`str.at`, `vec.check`, `vec.get`, `vec.get_mut`) have
~1,800 textual uses in `self/`+`std/`, and `vec.get` inlines `vec.check`, so lowering
runs `thir.check_fn` 4,000 to 7,000 times on the same handful of bodies; each also
rescans the whole emitted body in `next_slot`. Expect a visible cut to both `check` and
`build`; the exact share shows in `time|check` after Phase 1.

Files: `self/inline.llr` (129 lines), `self/lower.llr` (496 lines: **split first**),
`self/lower_call.llr` (275), `self/lower_clos.llr` (106), callers of `lower.lower_fn`
(`self/check.llr`, `self/drive.llr` x2, `self/wasm_fn.llr`, `self/lower_clos.llr:19`).

Design.

- Memo. A cache struct in `inline.llr`:
  `(pub (struct icache (keys (vec str)) (vals (vec thir.ex))))` keyed by
  `fn_path ++ "|" ++ ty.show` of each targ (there are four inline defs and at most a few
  hundred distinct instantiations, so a `(map str i64)` or a linear `str.eq` scan over
  `keys` both work; use the map). `try_inline` looks up, and on a hit **clones** the
  `thir.ex` out of the cache and calls `subst` by value as today (the bodies are a few
  nodes; cloning is cheap and avoids borrowing from `w` while mutating `w`). On a miss it
  runs `check_fn`; on `res.ok` it inserts a clone and proceeds; on `res.err` it does what
  it does today (append `sink`, do not cache). `lower_clos.clone_ex` (line 90) is
  already a THIR deep clone; make it `pub` or move it next to `thir.ex`, do not write a
  second copy.
- Where the cache lives. `low` gets a field `(ic inline.icache)`. `lower.lower_fn` gains
  a parameter `(ic (&mut inline.icache))`, moves it into `w.ic` at entry with
  `mem.replace` and moves it back at exit. `lower_clos.llr:19` passes `(&mut (. w ic))`
  into the nested `lower_fn`. Every top-level caller owns one `icache` for the whole run:
  `check.run` (one for the loop, returned or threaded alongside `kept`), `drive.build`
  (one for `pump` and `tests`), `wasm_fn.build`. `inline.llr` already `use`s
  `proj.lower` and `lower` reaches `inline` through `lower_call`, so the module cycle
  already exists; `lower.llr` adding `(use proj.inline)` is fine.
- `next_slot` (lines 28-35). Add `(tslot i64)` to `low`, initialized 0; `open_trace`
  uses `w.tslot` as the slot and sets `tslot = slot + 1`. Today `next_slot` returns
  `max(strace.local) + 1` over all statements emitted so far, which equals the running
  counter provided no `strace` statement is ever removed or renumbered after being pushed.
  Verify with `rg "vec.pop|vec.truncate|vec.remove|vec.set" self/lower*.llr self/inline.llr`
  on `blocks`/`stmts`: if nothing rewrites emitted statements, the counter is exact. If
  something does, keep `next_slot` and stop at the memo.
- `lower_call.into_call` (line 28) clones every argument (`clone_ops`) before
  `try_inline` decides. Move the eligibility test (`e.id == 0`, `d.inline`, not
  `on_stack`) into a `pub` predicate `inline.wants` and clone only when it is true. MIR
  is unchanged because the clone was only ever consumed on the inline path.

Equivalence. `check_fn p id targs` depends only on `p`, `id`, `targs`; a cached
success is the same value `check_fn` would return again. Slot numbers are argued above.
`check.ps1`, `mir.ps1`, and `x64_ref.ps1` verify all three directly (`strace` slots are
printed by `emit-mir`).

Steps:

1. `[ ]` Split `lower.llr` (496) before touching it: e.g. `remap_locals`, `has_i`,
   `drop_params`, and the scope helpers into `self/lower_scope.llr`, or whatever cut
   keeps both files under 450. Byte-equal, separate step within the phase.
2. `[ ]` `icache`, thread the parameter, `tslot`, late clone.
3. `[ ]` Full **Verification**; measure; fill the row.
4. `[ ]` Commit: `self: inline callees are checked once per instantiation`.

### Phase 4: indexed `prog` (OPT 5, OPT 6) `[ ]`

Expected: `prog.meth_id` scans every method in every `impl` (M compares) for every
`needs_drop` of a struct, and `needs_drop` is called per temporary, per `let`, per `set`,
per drop statement, per liveness pass per drop access, per codegen slot and field. Every
name lookup (`item_id`, `use_mod`, `pre_id`, `mod_fn`) is a linear `str.eq` scan, and
`std_fn` builds a string and scans all modules per `panic`/`assert`/checked-division
site. Expect cuts across `check`, borrowck, and codegen; the `time|` split tells which.

Files: `self/prog.llr` (371 lines), new `self/prog_idx.llr` (builds the tables),
`self/collect.llr` (485 lines: **only the one call** that builds the tables; do not grow
it), `self/resolve.llr` or `self/load.llr` wherever `collect` finishes (find it with
`rg "blank_prog|add_meth" self`).

Design. New fields on `prog.prog`, all filled by `prog_idx.build (&mut prog)` once after
collection and never written again:

| field | type | replaces |
| --- | --- | --- |
| `modu.imap` | `(map str i64)` | `item_id` scan over `names`/`ids` |
| `modu.umap` | `(map str i64)` | `use_mod` scan over `uals`/`umod` |
| `prog.pmap` | `(map str i64)` | `pre_id` scan over `pn`/`pi` |
| `prog.mmap` | `(vec (map str i64))` by owner def id | `meth_id` scan over `mo`/`mn`/`mi` |
| `prog.modmap` | `(map str i64)` module path to index | `std_fn` scan over `mods` |
| `prog.hasdrop` | `(vec bool)` by def id | `meth_id p id "drop" >= 0` inside `needs_drop` and `mir.can_move_field` |
| `prog.dropk` | `(vec i8)` by def id: 1 always, 0 never, -1 depends on args | the `field_drop` recursion for adts whose fields mention no type parameter |

- **First-wins.** Every linear scan today returns the first match. Build each map with
  `(if (not (map.has ...)) (map.put ...))` so a duplicate name resolves to the same id it
  does today. Keep the parallel vectors; they are still the iteration order for anything
  that enumerates (and `add_item`/`add_meth` keep pushing to them during collection).
- `lookup_qual`/`lookup_fn2`/`lookup_fn3` still `str.slice` the dotted name twice; with
  the maps that is two small allocations per lookup instead of several scans. Leave it.
  If `time|check` still shows name resolution after this phase, add a per-module
  `(map str i64)` of **full dotted path to fn id** filled in `prog_idx.build` by walking
  every def body once with today's `lookup_fn`, and make `lookup_fn` consult it first.
  That is exactly equal by construction and costs one walk.
- `dropk`. For every adt def, run today's `needs_drop` logic symbolically: a field type
  of kind 13 (param), or an adt whose args contain a param, makes the def `-1`; kinds 10
  and 15 make it `1`; kind 8 recurses on the field's def (`hasdrop` first); everything
  else contributes `0`. Recursion terminates because by-value field nesting is acyclic
  (a self-containing struct has no finite layout; `vec`/`box`/`ptr`/`&` stop the walk),
  but guard with a `visiting` vector anyway and treat a cycle as `-1`. `needs_drop` then
  is: kinds 10/15 `true`; not 8 `false`; `hasdrop[id]` `true`; `dropk[id] >= 0`
  `(== dropk 1)`; else today's `field_drop`.

Equivalence. Each map returns the first-wins id of the scan it replaces. `hasdrop` is
`meth_id p id "drop" >= 0` evaluated once; `dropk` is `needs_drop` evaluated once for
defs whose answer cannot depend on `args`. `std_fn` with `modmap` returns the same module
index because module paths are unique (`load_one` dedups by path, OPT **Already fine**).
`check.ps1` and `mir.ps1` cover the frontend uses; `x64_ref.ps1` covers `cg_frame`,
`cg_drop`, and `mir.can_move_field`.

Traps:

- `prog.def_at` and `blank_prog`/`blank_mod` construct these structs positionally; every
  new field needs a default there and anywhere else `prog`/`modu` is built
  (`rg "\(prog \(|\(modu " self`).
- `thir_call.check_call` tries `thir.lookup` -> `lookup_fn` -> `lookup_type` ->
  `builtin_id` -> `call_trait` in sequence (OPT 6). Do not reorder; the maps make each
  step cheap.

Steps:

1. `[ ]` `prog_idx.build`, the fields, the constructors, the map-backed lookups.
2. `[ ]` `needs_drop` via `hasdrop`/`dropk`; `mir.can_move_field` via `hasdrop`.
3. `[ ]` Full **Verification**; measure; fill the row.
4. `[ ]` Commit: `self: prog name, method, and drop tables are indexed`.

### Phase 5: borrow checker dataflow (OPT 4) `[ ]`

Expected: liveness is a backward problem iterated in forward block order, so a
straight-line chain of `nb` blocks (one per call) takes about `nb` passes, each pass
allocating two n-sized vectors and a fresh `stmt_acc` per statement; conflicts are
O(S x L x n). Expect a cut to `check` proportional to the borrowck share in `time|check`
after Phase 4; measure that share first and skip sub-items that do not show.

Files: `self/liveness.llr` (118 lines), `self/borrow.llr` (261), `self/holders.llr`
(419: split before growing), `self/moves.llr`, `self/loans.llr`, `self/mir.llr`
(`succs`).

Design, in order of payoff:

1. **Hoist access lists.** Compute `stmt_acc`/`term_acc` once per body into
   `(vec (vec (vec borrow.acc)))` (block, point) and pass `(& ...)` to `liveness.transfer`,
   `liveness.points`, `conf_blk`, and `holders`/`moves` where they recompute them.
   `acc` owns two small vectors per place; today each is rebuilt per pass.
2. **Liveness order and scratch.** `solve` visits blocks in reverse post-order (compute
   once from `mir.succs`) with a worklist seeded with all blocks; a block is re-queued
   when its `inn` changes, for each predecessor (build `preds` once). `apply` stops
   allocating `uses`/`defs`: first clear every def access's local in `live`, then set
   every use access's local; same result as `(live - defs) | uses`.
3. **Per-local `needs_drop` once.** `gen_one` kind 5 calls `prog.needs_drop` per drop
   access per pass; precompute `(vec bool)` over `b.locs` once per body and index it.
   Same for any `needs_drop` inside `holders`/`moves` per pass.
4. **Conflicts.** `conf_loan` is called per loan per point; `in_force` and `after` each
   scan all `nloc` locals. Compute per point one `(vec bool)` over loans, "some live
   holder", by iterating only the locals live at that point (from `liveness.live_at`)
   and OR-ing their held loans; change `holders.idx` to `local * nloans + loan` so a
   local's loans are contiguous. `in_force` then reads one bit. `after` needs the same
   over `live_after`. The pick order in `holder_name` (lowest local index first) must
   stay, since it chooses the name in the diagnostic.
5. **Packed bits.** `borrow.bools/bit/setb/clone_b` and the `union_b`/`diff` in
   `liveness` over `(vec bool)` become `(vec u64)` words. Only if items 1 to 4 leave
   borrowck visible in `time|check`; it touches every dataflow file.
6. `mir.succs` allocates a vector per call; a `(vec (vec i64))` per body computed once
   serves `liveness`, `lint.unreachable`, and `holders`.

Equivalence. A monotone dataflow fixpoint is unique, so visiting order and worklist
discipline cannot change `inn`/`outv`. Hoisted `acc` lists are the same values
`stmt_acc` produced per pass. The per-point loan bitset is `exists i: held and live`,
which is what `in_force` computes by scanning. `check.ps1` on `tests/err` (every
`borrow.*`, `move.*`, `init.*` golden) is the direct test; `mir.ps1` is unaffected
(borrowck emits no MIR); the suite compiles every std and tool file.

Traps:

- `holders` and `moves` are forward problems already iterated forward and converge in
  about loop depth plus one passes (OPT **Already fine**); do not reorder them, only
  stop their per-pass allocation.
- `points` and `rev` in `liveness` produce the per-statement sets the conflict pass
  indexes by `(bb, idx)`; keep the indexing, drop the double clone (build `pts` in
  reverse directly or fill a pre-sized vector).

Steps:

1. `[ ]` Items 1 to 3; verify; measure `time|check`.
2. `[ ]` Item 4 if borrowck is still visible; verify; measure.
3. `[ ]` Items 5 and 6 only on evidence.
4. `[ ]` Full **Verification**; fill the row.
5. `[ ]` Commit: `self: liveness worklist, hoisted accesses, per-point loan sets`.

### Phase 6: checker and codegen tables (OPT 7, 8, 9, 10) `[ ]`

Expected: `thir.shallow` clones the whole type at every chain step, twice per `unify`;
`cg_mod.find_fn` is a linear scan over all instantiations per call terminator and twice
per fn; `cg_lay.of` is a linear `ty.eq` plus a deep clone per projection step; string,
vtable, and drop-glue tables are linear scans and quadratic under `--checked`. Expect a
cut to codegen time (`time|codegen`) and some `check`.

Files: `self/thir.llr` (492 lines: **split first**), `self/ty.llr` (242; adds `hash`),
`self/cg_mod.llr` (355), `self/cg_lay.llr` (200), `self/cg_frame.llr` (415), `self/cg_clos.llr`,
`self/cg_ovf.llr`, `self/lower_div.llr`, `self/drive.llr` (`drain`/`inst` by index).

Design.

- **`shallow` clones once, not per step.** Today (`thir.llr` 153-158) `cur` is a clone of
  `t` and is re-cloned from `infer` at each hop. Walk by reference instead: `(let mut cur t)`
  as `(& ty.sty)`, hop to `(vec.get (& (. c infer)) id)` while `kind == 14u8` and
  `iset[id]`, and return `(ty.clone cur)` at the end. The returned value is the same
  `sty` (the first in the chain that is not an unresolved inference variable), so every
  caller is unchanged. **Do not** return `(& ty.sty)` from `shallow`: `unify` calls
  `shallow` twice and then `set_infer` with `(&mut ck)`, which the borrow checker rejects
  while the returned references are live (loans on `c` through the return value,
  `eligible_args` rule). Do not add path compression: `set_infer` stores the exact `sty`
  passed, and the stored `flag` on a chain link can differ from `iint[j]` (see
  `unify_both`), so rewriting links could change a later `shallow` result. The rest of
  OPT 7 (`rest_block` loop-body clones, `vec.remove 0` in `append_ty`, `method_first`
  snapshots) is only worth touching if `time|check` still points at the checker.
- **Structural type hash.** `ty.hash ((& sty)) i64`: fold `kind`, `id`, `flag`, the
  bytes of `text`, and `args` recursively with the FNV mix `cg_mod.mix` uses. This is
  for **lookup only**. `cg_mod.mangle` keeps `hash_tys` over `ty.show` because the hex
  in every generic symbol name is part of the object bytes.
- **`find_fn` by index.** `cg` gains `(fmap (map i64 (vec i64)))` from
  `mix(id, hash of args)` to candidate indices, verified with `args_eq`. `func_sym`
  returns the symbol as today but `pend` becomes a list of **instance indices** (not
  ids), `pargs` goes away, and `drain`/`inst` read `fid[i]`/`fargs[i]`, so `is_done` and
  `mark_done` are `fdone[i]`. `cg_mod.done_ids` is unchanged. `wasm_mod` has the same
  shape (`wasm_mod.is_done`/`mark_done`); do it there too or leave wasm on the scan
  (wasm is not measured; leaving it is acceptable, say so in the log).
- **`cg_lay.of` by index.** `lays` gains `(lmap (map i64 (vec i64)))` keyed by `ty.hash`,
  verified with `ty.eq`. Add `of_idx` returning the index and `size_at`/`align_at`/
  `field_at`/`var_at` readers; move the hot callers in `cg_frame.field_step`,
  `local_slot`, aggregate copy, and `store_rec` to the readers. Keep `of` for the rest.
  `cg_clos.fill`/`emit_call`/`drop_caps` build a fresh `cg_lay.new` each time; pass
  `(&mut (. g lays))` if the borrow allows, else leave and note it.
- **Keyed tables.** `skey`/`vkey` gain `(map str i64)` beside the vectors (the vectors
  keep numbering `llr_sb<n>`, `llr_ss<n>`, `llr_k<n>`); `dty` gains a `ty.hash` map with
  `ty.eq` verification. Insert into the map wherever the vector is pushed (`str_sym`,
  `scalar_sym`, `vt_put`, `drop_sym`). `find_key` and `find_vt` share `vkey`; one map
  serves both.

Equivalence. `shallow` returns the same value. Hash maps verified with `args_eq`/`ty.eq`/
`str.eq` return the same index the scan returned, provided insertion order into the
vectors is unchanged (it is; the maps are added beside the pushes). Symbol names are
untouched. `x64_ref.ps1` is the direct test for all of codegen; `check.ps1`/`mir.ps1`
for `shallow`.

Steps:

1. `[ ]` Split `thir.llr` under 450 (e.g. the `ck` scope helpers or the `zonk`/`unify`
   family into `self/thir_unify.llr`). Byte-equal, separate step.
2. `[ ]` `shallow`; verify with `check.ps1`/`mir.ps1`.
3. `[ ]` `ty.hash`, `find_fn`/`pend` by index, `cg_lay` index, keyed tables; verify with
   `x64_ref.ps1`, including one `--checked` build of `self/llrs.llr --emit-obj` compared
   against the baseline compiler's output (the snapshot has no `--checked` entries; do it
   by hand once and record the two hashes in the log).
4. `[ ]` Full **Verification**; measure; fill the row.
5. `[ ]` Commit: `self: shallow clones once, hashed instance, layout, and constant tables`.

### Phase 7: link probing and object assembly (OPT 11, OPT 12) `[ ]`

Expected: linking is ~4 s on Windows regardless of program size, which is most of a
`hello` build after the phases above. Object assembly is linear but pushes every byte of
a 6 MB `.text` three times through bounds-checked `vec.push`.

Files: `self/link.llr` (438 lines; fold before adding), `self/link_win.llr` (108),
`self/cg_mod.llr` (`append_x`/`append_a`), `self/coff.llr`, `self/elf.llr`,
`self/elf_out.llr`.

Design.

- **Probe once per build.** `via_win` calls `have ".o"` and `have ".obj"` (8 `runtime`
  lookups, each `os.env` x2 and up to five `fs.exists`), then `link_win.pick` spawns
  `clang --version`, `gcc --version`, `cc --version`, `cl /nologo /?`, and `vswhere.exe`;
  then `via_cl` -> `link_win.link` -> `prefix` spawns `has_cl` and `bat` **again**.
  Compute `prefix` once in `pick` (return a small struct `{tool, prefix}` or a second
  out value) and pass it into `link_win.link`; compute the `runtime` paths once and pass
  them down. Add an `LLRST_CC` override (`cl`, `clang`, `gcc`, `cc`) that skips
  `first_gnu`/`cl_ready` entirely; document it in `docs/spec/targets.md` if that is
  where toolchain env vars live (check `rg LLRST_RUNTIME docs`).
- **`vcvars64.bat`.** It runs only when `cl` is not already on `PATH` (`prefix` returns
  empty when `has_cl`). A Developer PowerShell avoids it. Caching the environment it
  produces to a file would survive the process and is **rejected** by the rules above;
  the right fix is to say in `docs/install.md` that builds are faster from a Developer
  prompt, and to let `LLRST_CC=cl` skip the probes.
- **Symbol wants without scanning the bytes.** `wants_ui`/`wants_web`/`want_sym` each
  `str.has` over the whole object. `drive.build` has `g.obj` before `cg_mod.finish`;
  collect the undefined symbol names once (`elf.obj` symbol table) into a `(vec str)` and
  pass it to `link.link` as a `needs` parameter. Equivalent because every name those
  scans look for is an undefined symbol the object references.
- **Object assembly.** `append_x`/`append_a`: `vec.reserve` then a bulk copy from the
  ISA buffer (`x64_buf`/`a64_buf` expose `len`/`at`; add a `bytes` accessor returning
  `(& (vec u8))` and use `vec_more.extend`, or `sys.memcpy` via `vec.ptr` if `extend`'s
  per-element `Clone.clone` shows). `coff.chunk`/`put_bytes` and `elf_out.clone_bytes`
  the same way; `coff.to_str` and `elf_out.to_str` use `str.from_bytes`
  (`std/str_more.llr`) after `str.reserve`. `emit_rels` three passes stay unless the
  timer says otherwise.

Equivalence. Bulk copies produce the same bytes. The link command line is the same
string; only how often the probes run changes. `x64_ref.ps1` covers assembly; the
`stage2=equal` hash and `run.ps1` cover the link.

Steps:

1. `[ ]` Probe once, `LLRST_CC`, `needs` list; measure `time|link` on `hello`.
2. `[ ]` Bulk copies; measure `time|object` on `self/llrs.llr`.
3. `[ ]` Full **Verification**; measure; fill the row.
4. `[ ]` Commit: `self: toolchain probed once, LLRST_CC, bulk object copies`.

### Phase 8: close out `[ ]`

Files: this file, `OPT.md`, `HANDOFF.md`, `README.md`, `history/`.

1. `[ ]` Re-measure everything in **Measuring** with the final `llrs0` and with `llrs1`
   (to confirm the `llrs1`/`llrs0` ratio is still ~1.1, i.e. the backend is still not
   the bottleneck). Append the final table to `OPT.md` under **Measurements** with the
   commit hash; do not edit OPT's findings text.
2. `[ ]` For every OPT item 13 and 14 sub-bullet, either do it now if a timer shows it
   (each is a one-line `vec.append`/`vec.reserve` change) or write one line here saying
   it was not visible.
3. `[ ]` OPT 15 (register allocation) stays where it is: `HANDOFF.md` Phase 15 item 1.
   The measured `llrs1`/`llrs0` ratio is the evidence that it is a ~10% lever for
   compile time, not the 55x.
4. `[ ]` Move this file to `history/comp.md` with every box checked, add it to the
   finished-plans list at the top of `HANDOFF.md` and to the `README.md` table, and
   update the `HANDOFF.md` **Status** block (`self|...`) with the new build time.
5. `[ ]` Commit: `docs: compile-speed plan closed, measurements recorded`.

## Decisions (made for this plan; do not reopen without a measurement)

| topic | decision |
| --- | --- |
| scope | `self/` only. No std, language, or stage 0 change for speed. |
| invariant | Every phase is output-byte-equal: `check.ps1`, `mir.ps1`, `x64_ref.ps1`, `stage2=equal`, `run.ps1`. |
| measurement binary | `llrs0` (built by stage 0), `self/llrs.llr` and `examples/hello.llr`, Windows, `SOURCE_DATE_EPOCH=0`. |
| caches | Per-process only. No files beside the exe, no cached toolchain environment. |
| `prog` tables | Built once in `prog_idx.build` after collection; `prog` stays `&` everywhere else. |
| `shallow` | Returns an owned `sty` (one clone). Not a reference (borrowck), no path compression (`flag` vs `iint` drift). |
| `mangle` | Keeps `hash_tys` over `ty.show`; `ty.hash` is for lookups only. |
| OPT 15 | Out of scope here; scheduled in `HANDOFF.md` Phase 15. |
| order | 0 harness, 1 lint, 2 bodies, 3 inline, 4 prog, 5 borrowck, 6 tables, 7 link/object, 8 close. Re-order only on a `time|` measurement. |

## Progress log

- 2026-11-05: Plan written from `OPT.md` (findings at `7077aa6`). Line citations in OPT
  re-checked against the tree at `1cb42f5` and found current. No code changed. Suite not
  run. Observations recorded while reading, for whoever starts Phase 1: `diag.flush` sorts
  before printing, so moving a diagnostic between passes cannot reorder stderr;
  `lint.items` is the likely majority of the 244 s `check` (private fns x whole-program
  AST x `lookup_fn`); `std/map.llr` is the hash map and `self/` does not use it yet;
  `lower.llr` (496), `thir.llr` (492), `thir_call.llr` (499), `thir_expr.llr` (495), and
  `collect.llr` (485) are at the 500-line cap and must be split before growing.
