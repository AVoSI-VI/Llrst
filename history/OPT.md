# Compile-time findings (archive)

This was `OPT.md` until 2026-11-06. It is the evidence record for
[comp.md](comp.md). The live handoff is [../HANDOFF.md](../HANDOFF.md).

# Llrst: compile-time findings for `self/`

Where the self-hosted compiler (`llrs`, installed as `llrc`) spends its time, and what
would make it faster. Scope is `llrs` compiling `.llr` programs; the Rust stage 0 is only
the baseline. Nothing here has been implemented; every item is a candidate, cited to the
code as of commit `7077aa6` (2026-11-04). When code and this file disagree, trust the code.

## Measurements

Windows 11, 8 logical CPUs, MSVC `cl` as the link driver, `SOURCE_DATE_EPOCH=0`.
Stage 0 is a fresh `cargo build --release` of `compiler/`. `llrs0` is stage 0's output,
`llrs1` is `llrs0`'s. One run each; expect a few percent of noise.

| compiler | command | input | wall time |
| --- | --- | --- | --- |
| stage 0 (Rust, Cranelift) | `check` | `self/llrs.llr` | 0.69 s |
| stage 0 | `build` (link) | `self/llrs.llr` | 6.66 s |
| `llrs0` | `check` | `self/llrs.llr` | 244.38 s |
| `llrs0` | `build --emit-obj` | `self/llrs.llr` | 344.42 s |
| `llrs0` | `build` (link) | `self/llrs.llr` | 355.36 s |
| `llrs1` | `build` (link) | `self/llrs.llr` | 390.12 s |
| `llrs0` | `check` | `examples/hello.llr` | 1.62 s |
| `llrs0` | `build --emit-obj` | `examples/hello.llr` | 1.79 s |
| `llrs0` | `build` (link) | `examples/hello.llr` | 5.96 s |
| stage 0 | `build` (link) | `examples/hello.llr` | 4.14 s |

Input size: `self/` is 92 files, 17,031 lines, ~3,600 fn defs with `std/` (9,183 lines).
Output: `llrs_t.obj` is 5.96 MB, `llrs.exe` 5.28 MB.

What the numbers say:

- `llrs` is about 55x slower than stage 0 on the same input (355 s vs 6.7 s). The
  bootstrap `scripts/self.ps1` therefore costs ~12.5 minutes of `llrs` time plus the suite.
- `check` alone is 70% of a build (244 of 344 s). `check` is `check.run` (typecheck,
  lower, borrowck, lint per user fn, then `lint.items`) and nothing from codegen.
  Everything in `check.run` is repeated inside codegen (item 1 below), so the extra
  100 s of `build --emit-obj` includes a second pass over every user fn, all `std`
  fns, all generic instances, and x86-64 emission. The second pass being cheaper than
  the first points at `lint.items` (item 2), which only runs in `check.run`.
- Linking is ~4 s on Windows regardless of program size (`hello`: 1.8 s to the object,
  6.0 s to the exe). That is toolchain probing plus `vcvars64.bat` (item 11).
- `hello.llr` already pays 1.6 s to `check` one function; that is std load, parse,
  collect, and `lint.items` fixed cost.

No profiler was run. The per-item impact estimates below come from reading the code
and from the phase split above; the ordering is a judgement, not a measurement.

### After phases 0-7 (`6190934`, 2026-11-06)

Same host and `SOURCE_DATE_EPOCH=0`. `llrs0` and `llrs1` were rebuilt by
`scripts/self.ps1 --stage` (`self|stage2=equal`). Each command was run twice; the table
is the second run. Phase splits are in [history/comp.md](history/comp.md).

| compiler | command | input | wall time |
| --- | --- | --- | --- |
| `llrs0` | `check` | `self/llrs.llr` | 14.41 s |
| `llrs0` | `build --emit-obj` | `self/llrs.llr` | 17.47 s |
| `llrs0` | `build` (link) | `self/llrs.llr` | 19.00 s |
| `llrs0` | `build` (link) | `examples/hello.llr` | 1.63 s |
| `llrs1` | `check` | `self/llrs.llr` | 17.62 s |
| `llrs1` | `build --emit-obj` | `self/llrs.llr` | 20.46 s |
| `llrs1` | `build` (link) | `self/llrs.llr` | 21.58 s |
| `llrs1` | `build` (link) | `examples/hello.llr` | 1.71 s |

`llrs0` `build` of `self/llrs.llr` is 19.00 s against 355.36 s above. `llrs1` / `llrs0`
on that build is 1.14 (21.58 / 19.00); on `check` it is 1.22 (17.62 / 14.41).
Second-run `llrs0` `time|` splits for that build, in seconds: load 0.64, check 13.92,
codegen 3.00, object 0.05, link 1.31. Hello `build` is load 0.20 and link 1.33.

## Findings, highest estimated impact first

Notation: N = fn defs in the program (~3,600), F = instantiated fns, C = call sites,
S = MIR statements in one fn, nb = blocks in one fn, n = locals, L = loans, M = methods
in all `impl`s, T = distinct types seen by layout, K = string literals.

### 1. Every user fn is typechecked, lowered, and borrow-checked twice

`drive.build` calls `check.run` first ([self/drive.llr](self/drive.llr) line 85), which
runs the full frontend for every non-std, non-generic, non-extern fn:

```
self/check.llr:20   (for i 0 (vec.len (& (. p defs))) (if (user_fn p i) (one p out i dump want_mir)))
self/check.llr:33   (match (thir.check_fn p id empty out)
self/check.llr:43   (let b (lower.lower_fn p e (prog.fn_path p id) out))
self/check.llr:44   (borrow.run p out (& b))
```

Then `pump` -> `drain` -> `inst` does it all again for every reachable `(id, targs)`,
including those same fns with empty `targs`; the THIR and MIR from `check.run` are
discarded:

```
self/drive.llr:232  (match (thir.check_fn p id (clone_tys (& args)) ds)
self/drive.llr:236  (let b (lower.lower_fn p e (prog.fn_path p id) ds))
self/drive.llr:237  (borrow.run p ds (& b))
self/drive.llr:238  (if (not (check.user_fn p id)) (lint.body ds (& b)))
```

Line 238 skips only `lint.body` on the second pass. `self/wasm_fn.llr` line 69 has the
same second `check_fn`. Cost: roughly 2x on items 3 to 8 for the ~2,500 fns in `self/`.
Fix: keep the `mir.body` per def from `check.run` (a `vec` indexed by def id on `prog`
or passed into `cg_mod`) and have `inst` use it when `args` is empty; or skip the
per-fn loop in `check.run` when `cmd` is `build` and let `inst` carry the lints.

### 2. `lint.items`: unused-fn and unused-use checks are quadratic in program size

For every private user fn, `called` walks every def body in the whole program and runs
`prog.lookup_fn` (two `str.slice` allocations plus linear name scans, item 6) on every
name and call node:

```
self/lint.llr:152   (if (or (. d is_pub) (or (. d extern) (called p i test))) (ret))
self/lint.llr:164   (fn called ((p (& prog.prog)) (id i64) (test bool)) bool
self/lint.llr:165     (for i 0 (vec.len (& (. p defs)))
self/lint.llr:181   (fn refs_e ((p (& prog.prog)) (mi i64) (e (& expr)) (id i64)) bool
self/lint.llr:182     (if (and (or (== (. e kind) 1u8) (== (. e kind) 23u8)) (== (prog.lookup_fn p mi (& (. e text))) id))
```

Cost: O(private fns x total AST nodes x lookup_fn), with ~1,600 private fns in `self/`;
local variable references (kind 1) also go through `lookup_fn`. `use_seen` has the
same shape per module x per `use` alias (lines 120-125), and `has_alias` allocates a
new string per visited node (lines 143-146). Fix: one pass over all bodies that
resolves each name once and marks the referenced def id and the alias prefix, then
report unreferenced fns and aliases in O(N).

### 3. Inline callees are re-typechecked at every call site; `next_slot` rescans the body

```
self/inline.llr:17   (let body (thir.check_fn p (. e id2) (ty.clone_tys (& (. e tys))) (&mut sink)))
self/inline.llr:28   (fn next_slot ((w (& lower.low))) i64
self/inline.llr:30     (for bi 0 (vec.len (& (. w blocks)))
self/inline.llr:32       (for k 0 (vec.len (& (. blk stmts)))
```

The only `inline` fns are `str.at`, `vec.check`, `vec.get`, `vec.get_mut`
(`std/str.llr` line 11, `std/vec.llr` lines 28-33). `self/` and `std/` have ~1,800
textual uses, and `vec.get` inlines `vec.check` inside it, so each `vec.get` site is two
`check_fn` runs; with item 1 that is on the order of 4,000 to 7,000 extra `check_fn`
calls. `next_slot` rescans every statement emitted so far per inline site, so a fn
with k `vec.get`s is O(k x S). `lower_call.into_call` also deep-clones all args
(`clone_ops`, `self/lower_call.llr` lines 28, 83-86) before knowing the callee is
inline. Fix: memoize the checked THIR per `(callee id, targs)` on `lower.low` or
globally; keep a running max trace slot on `low` instead of `next_slot`; pass `args`
by reference and clone only on success.

### 4. Borrow checker: liveness iterated in the wrong order with per-pass allocation

Backward dataflow, blocks visited forward:

```
self/liveness.llr:34   (while changed
self/liveness.llr:36     (do (set changed false) (for bb 0 nb (if (solve_bb p b n nb inn outv bb) (set changed true))))))
```

`lower.new_block` numbers blocks in creation order and every call opens a new block
(`self/lower_call.llr` lines 29-32), so a straight-line chain of nb blocks needs about
nb passes. Each pass, per statement, `apply` allocates two n-sized vectors and a fresh
`stmt_acc` (lines 91-96, `self/borrow.llr` lines 36-44, 88-96), and `gen_one` calls
`prog.needs_drop` (item 5) per drop access (lines 114-116). After solving, `points`
re-runs the transfer and `rev` clones every per-statement set again (lines 71-89).
Cost: about O(nb x S x n) time and O(nb x S) heap allocations per fn.

Conflict detection is O(S x L x n): `conf_at` loops every loan per statement
(`self/borrow.llr` lines 168-178) and `holders.in_force` scans all locals per loan
(`self/holders.llr` lines 19-23), again in `holders.after` (lines 40-53). `holders.record`
snapshots an L x n bitset per statement via `vec.clone` (lines 390-407); `kill_local`
and `append_pairs` are O(L) per operand (309-316); `loan_at` is a linear scan per
reference statement (291-295); `on_term` recomputes `loans.eligible`/`loans.sinks` with
`ty.subst`/`ty.lts` allocations on each pass (330-346, `self/loans.llr` 62-71, 104-114).
`moves.drop_local` rebuilds four vectors per whole-local assignment
(`self/moves.llr` 59-70) and `step` clones state per block per pass (138-146).

Fix: iterate liveness in reverse post-order with a worklist; compute `stmt_acc` once per
statement and keep it; reuse `uses`/`defs` scratch; precompute per point the set of
loans held by any live local once instead of per loan x access; index loans by
`(bb, idx)`; compute `eligible`/`sinks` once per call terminator; pack bitsets.

### 5. `prog.needs_drop` -> `prog.meth_id` scans every method in the program, uncached

```
self/prog.llr:244   (pub (fn meth_id ((p (& prog)) (owner i64) (name (& str))) i64
self/prog.llr:245     (for i 0 (vec.len (& (. p mo)))
self/prog.llr:363   (pub (fn needs_drop ((p (& prog)) (t (& ty.sty))) bool
self/prog.llr:366     (if (>= (meth_id p (. t id) "drop") 0) (ret true))
self/prog.llr:367     (field_drop p t)))
```

`mo`/`mn`/`mi` are flat parallel vectors over all `impl` fns. For a struct without
`drop` the scan fails after M compares, then `field_drop` (349-361) allocates a
`ty.subst` and recurses per field; `cg_mod.cg` alone has ~30 `vec` fields. Hot callers:
`lower.temp` per temporary, `stmt_let`, `stmt_set` (`self/lower.llr` 129, 168, 272-281),
`lower_match` (50, 99), `liveness.gen_one` per pass, `cg_frame.local_slot` per local and
`field_mask`/`field_mode` per field (`self/cg_frame.llr` 102-138), `cg_drop.place` per
drop statement, `mir.can_move_field` (`self/mir.llr` 226). `meth_id` is also on the
name-resolution path (`lookup_fn2`, item 6). Fix: index methods by owner (`vec (vec i64)`
by def id, or a `std.map`) and memoize `needs_drop` per `(def id, args)`, or precompute a
per-def "has drop glue" bit in `collect`.

### 6. All symbol tables in `prog` are linear `str.eq` scans; dotted names allocate per lookup

```
self/prog.llr:199   (fn find_name ((names (& (vec str))) (ids (& (vec i64))) (name (& str))) i64
self/prog.llr:200     (for i 0 (vec.len names) (if (str.eq (vec.get names i) name) (ret (@ (vec.get ids i)))))
self/prog.llr:270   (fn lookup_fn2 ((p (& prog)) (module i64) (path (& str))) i64
self/prog.llr:272     (let a (str.slice path 0 dot))
self/prog.llr:273     (let b (str.slice path (+ dot 1) (str.len path)))
```

`item_id`, `use_mod`, `pre_id` (203-207) are parallel vectors scanned per lookup;
`lookup_fn` does `count_dots`, `str_dot`, two heap `str.slice`s, up to three scans,
then `lookup_type` and `meth_id`. `std_fn` (341-347) builds a `"std."+module` string
and scans all modules per call; it is called per `panic`/`assert`/`assert_eq` site
(`self/thir_pat.llr` 233, 249; `self/thir_call.llr` 441, 457), per checked division
(`self/lower_div.llr` 22), per closure fill and per trap. `thir_call.check_call`
(`self/thir_call.llr` 17-37) tries `thir.lookup` -> `lookup_fn` -> `lookup_type` ->
`builtin_id` (nine `str.eq`) -> `call_trait` in sequence, so a name that is not a plain
fn pays all of them. `std/map.llr` is a real chained hash map (lines 7-30); `prog` does
not use it. Fix: give `modu` a `(map str i64)` for items and aliases; a per-owner method
map; pre-split or intern dotted names in `collect`; cache `std_fn` results.

### 7. `thir.shallow` deep-clones the type on every call; no path compression

```
self/thir.llr:153   (pub (fn shallow ((c (& ck)) (t (& ty.sty))) ty.sty
self/thir.llr:154     (let mut cur (ty.clone t))
self/thir.llr:157       (set cur (ty.clone (vec.get (& (. c infer)) (. cur id)))))
self/thir.llr:191   (pub (fn unify ((c (&mut ck)) (a (& ty.sty)) (b (& ty.sty))) bool
self/thir.llr:192     (let sa (shallow c a))
self/thir.llr:193     (let sb (shallow c b))
```

`shallow` clones the whole type (args vec and text) at each chain step and is called
twice per `unify`, once per `coerce`, `finish_arg`, `autoderef` step, `field_of`,
`join`. `unify` recurses on `args`, so a nested type is cloned once per level,
O(size^2) per unify. `unify_both` (184-189) points `i -> j` with no compression, so
chains grow with the number of equated vars. Zonk is a single pass per `check_fn`
(429-437); that part is fine. Related: `rest_block` deep-clones loop bodies before
checking them (`self/thir_expr.llr` 224-227, `kid_owned` 94-102), once per enclosing
loop level per `check_fn`; `instantiate` -> `append_ty` moves fresh vars with
`vec.remove 0` (`self/thir_call.llr` 175-181); `method_first` snapshots seven vectors
when a call name is both a fn and an enum variant (85-103). Fix: return a reference or an
index from `shallow` and clone only when the caller needs ownership; compress chains in
`shallow`/`set_infer`; give `check_block` a `from` offset like `check_list` already has.

### 8. `cg_mod.find_fn`: linear scan over all instantiations, three times per fn and once per call site

```
self/cg_mod.llr:147   (fn find_fn ((g (& cg)) (id i64) (args (& (vec ty.sty)))) i64
self/cg_mod.llr:148     (for i 0 (vec.len (& (. g fid)))
self/cg_mod.llr:156   (pub (fn func_sym ((g (&mut cg)) (p (& prog.prog)) (id i64) (args (vec ty.sty))) i64
self/cg_mod.llr:189   (pub (fn mark_done ((g (&mut cg)) (id i64) (args (& (vec ty.sty)))) unit
self/cg_mod.llr:193   (pub (fn is_done ((g (& cg)) (id i64) (args (& (vec ty.sty)))) bool
```

`func_sym` runs it per `call_def` terminator (`self/cg_term.llr` 233), per drop-glue
method call, fn-pointer constant, and vtable slot: O(C x F). `drain` calls `is_done`
and `inst` calls `mark_done` (`self/drive.llr` 228, 231): O(F^2). `args` are cloned
twice in `func_sym` (163, 167-168) and twice more in `inst` (232, 240). A `hash_tys`
exists (113-116) but allocates via `ty.show`. Fix: a `std.map` from `(id, hash)` to
index; return the index from `func_sym` so `is_done`/`mark_done` are O(1); a
structural hash instead of `ty.show`.

### 9. `cg_lay.of`: linear `ty.eq` over all cached layouts, deep clone on every hit

```
self/cg_lay.llr:46   (fn find ((ls (& lays)) (t (& ty.sty))) i64
self/cg_lay.llr:47     (for i 0 (vec.len (& (. ls keys))) (if (ty.eq (vec.get (& (. ls keys)) i) t) (ret i)))
self/cg_lay.llr:52     (if (>= hit 0) (ret (clone_lay (vec.get (& (. ls vals)) hit))))
```

Called per projection step (`cg_frame.field_step`, line 391), per local slot (104),
per aggregate assign/copy/size, per `store_rec`. Each call is O(T) `ty.eq` plus
`clone_lay`, which allocates `fields` and every row of `vars`. `cg_clos.fill`,
`emit_call`, and `drop_caps` build a fresh `cg_lay.new` each time
(`self/cg_clos.llr` 65, 168, 190), bypassing the cache. `field_step` also does
`mir.field_ty` -> `ty.subst` per step (`self/cg_frame.llr` 397). Fix: hash the key;
return a reference or index (callers read `size`/`align`/`fields`/`vars`); share one
`lays` across closures; cache field types next to the layout.

### 10. String, vtable, and drop-glue tables are linear scans; quadratic under `--checked`

```
self/cg_mod.llr:203   (fn find_str ((g (& cg)) (s (& str))) i64
self/cg_mod.llr:204     (for i 0 (vec.len (& (. g skey))) (if (str.eq (vec.get (& (. g skey)) i) s) (ret i)))
self/cg_ovf.llr:10    (fn loc_str ((fx (& cg_frame.fncx))) str
self/cg_ovf.llr:19    (let ls (loc_str fx))
self/cg_ovf.llr:20    (let a (cg_mod.str_sym g (& ls)))
```

`str_sym` scans K strings per literal use. With `--checked`, every checked
`+ - * shl shr neg` emits a trap whose message is a unique `path|line|col` string, so
each misses after a full scan: O(K^2). `lower_div.loc_op` does the same per integer
division (`self/lower_div.llr` 37-38). `find_key`/`find_vt` (274-278, 300-303) and
`find_drop` (280-282, `ty.eq` per drop call site) share the pattern. Drop glue is
correctly deduplicated per type (`drop_sym`, `drain_drop`); only the lookup is linear.
Fix: `(map str i64)` for `skey`/`vkey`; a type-keyed map for `dty`; one location table
instead of one symbol per trap.

### 11. Windows link: toolchain probed twice, `vcvars64.bat` on every build

`link.link` -> `via_win` ([self/link.llr](self/link.llr) 369-373):

- `have` x2 -> `runtime` x8 (184-186, 30-45): `os.env` x2, `load.exe_dir`, four
  `path.join`s, up to five `fs.exists` each.
- `link_win.pick` -> `first_gnu` spawns `clang --version`, `gcc --version`,
  `cc --version` ([self/link_win.llr](self/link_win.llr) 50-59); `cl_ready` -> `has_cl`
  spawns `cl /nologo /?` and `bat` spawns `vswhere.exe` (25-39, 61).
- `via_cl` -> `runtime` x5 again, then `link_win.link` -> `prefix` -> `has_cl` and
  `bat` again (41-48, 84).
- Final command (86-107) is `call vcvars64.bat >nul && cl /nologo /Fe:... /link
  /STACK:8388608 /Brepro`; `vcvars64.bat` alone is typically 1 to 3 s.
- `wants_ui`/`wants_web`/`want_sym` each do a full `str.has` over the object bytes
  (190-192, 297, 301).

Measured: ~4 s per build on this machine, independent of program size. Fix: probe once
and cache (an env var such as a `LLRST_CC` override, or a file next to the exe); compute
`prefix` once and pass it to `link_win.link`; resolve `cl.exe`/`link.exe` absolute paths
from `vswhere` once and invoke directly without `vcvars64.bat`.

### 12. Object assembly copies the code three times, byte by byte

```
self/cg_mod.llr:317   (fn append_x ((g (&mut cg)) (sym i64) (c (& cg_isa.code))) unit
self/cg_mod.llr:320     (for i 0 (x64_buf.len b) (vec.push (&mut (. (. g obj) text)) (x64_buf.at b i)))
```

Then `coff.bytes` -> `chunk` -> `put_bytes` (`self/coff.llr` 40-43, 89-90) pushes
text/data/rodata/rels/syms one `vec.push` at a time, `to_str` (126-129) pushes every
byte again into a `str`, and `emit_rels` is three passes over all relocs (82-87). ELF
additionally `clone_bytes` the sections before `elf_out.assemble` (`self/elf.llr` 153,
166-168; `self/elf_out.llr` 78-92). Linear, but tens of millions of bounds-checked
calls for a 6 MB `.text`. Fix: `vec.reserve` plus `sys.memcpy`/`vec.append`; build the
final `str` with `str.reserve`.

### 13. Parsing: two allocations per token, repeated `forms` vectors

- `sexp.read` copies the source once (`std/sexp.llr` 122), fine. `peek` is re-evaluated
  up to four times per byte in `skip_ws` (32-33) and again in `adv` (26).
- One `str.slice` per atom/string/comment token (84, 100, 119), then `ast_expr` clones
  the text again per node (`std/ast_expr.llr` 53, 108, 129, 135).
- `ast_expr.forms` allocates a fresh `(vec i64)` per list node and is called up to three
  times on the same node for `(pub (fn ...))` (`std/ast.llr` 145, 343, 353).
- `same_std` calls `os.cwd` twice per std file (`self/load.llr` 127-130).

Low impact next to items 1 to 8; listed because `hello.llr` pays 1.6 s before any
codegen and most of that is load, parse, and collect of `std`.

### 14. Small quadratic idioms

- `while len>0: push (vec.remove v 0)` instead of `vec.append`: `self/load.llr` 161,
  `self/collect.llr` 275-279, 394, `self/thir_call.llr` 175-181, `self/thir_expr.llr` 454,
  `self/thir_clos.llr` 126-128, `self/inline.llr` 105, `self/mir.llr` 172. Small n.
- `lint.generics` -> `done_has` linear in `done` per generic def (`self/lint.llr`
  102-103, 200-202); `cg_mod.done_ids` builds a fresh vector each call (197-201).
- `inst` calls `diag.any_err ds ""` after every fn (`self/drive.llr` 239): O(F x diags).
- `cg_frame.set_flag` -> `row_copy` allocates per flag write (`self/cg_frame.llr`
  257, 293-296); `cg_frame.place` and `step` `ty.clone` per operand and per projection
  (368, 382).
- `lower_clos.into_clos` deep-clones the closure THIR before lowering
  (`self/lower_clos.llr` 50-56, 90-106).
- `mir.succs` allocates a vector per block per dataflow pass (`self/mir.llr` 211-220).
- `diag.append` clones each diag and `diag.sort` is an insertion sort that clones both
  elements per swap (`self/diag.llr` 72-82, 101-102); only matters with many diagnostics.
- `prog.impl_of` is a linear `ty.eq` scan (328-332); 39 impls today, cheap until it grows.
- `collect.add_timpl` dup detection is O(I^2) (`self/collect.llr` 261).

### 15. The compiler runs on its own output

`llrs` has no register allocator: every value goes through `acc`/`tmp`/`addr`/`dst`
and a frame slot (`self/cg_frame.llr`), every `vec.get` is an inlined bounds check plus
a load, and there is no constant folding or dead-store removal. Everything above is
therefore paid at an unoptimized-code constant factor. Stage 0's 0.69 s `check` of the
same input is the floor a good backend would approach. This is the largest lever
overall, and also the largest change; the algorithmic items above are reachable first.

## Already fine (do not re-report)

- Files are read and parsed once; `load_one` dedups by module path
  (`self/load.llr` 112-115); `fs.read` reads in doubling chunks (`std/io.llr` 13-22).
- Jump fixups: `x64_buf.label`/`bind`/`fixup`/`finish` are one O(fixups) pass with
  index-based labels (`self/x64_buf.llr` 245-281); always rel32, no relaxation.
- Relocations and symbols carry integer indices from `elf.add_sym`; there is no
  name-based symbol lookup in the back end. COFF string table is append-only.
- `vec.reserve` and `str.reserve` double; `str.push` is `memcpy`; `str.eq` is length
  plus `memcmp` (`std/vec.llr` 14-23, `std/str.llr` 14-47). `vec.new`/`str.new` do not
  allocate until first push.
- Native `sys.alloc`/`sys.grow` are libc `malloc`/`realloc`; the first-fit free list in
  `std/alloc.llr` is wasm-only.
- MIR blocks and statements are `vec`s of structs; no whole-body clones in codegen.
  Clone cost is in operands (`pk`/`pi` vectors), not bodies.
- `cg_frame.open` is linear in locals and blocks; `x64_abi.classify` is O(args).
- `holders` and `moves` are forward problems iterated forward and converge in about
  loop depth plus one passes; only liveness (item 4) has the wrong order.
- `cg_mod.is_done` prevents re-instantiating the same `(id, targs)`; only the lookup
  is linear (item 8). Drop glue is generated once per type (`ddone`).

## Suggested order

1. Item 1: reuse `check.run`'s bodies in `inst`. About 2x on the whole middle end,
   smallest diff.
2. Item 2: single-pass `lint.items`. Removes the only quadratic-in-N pass in `check`.
3. Item 3: memoize inline callee THIR; counter instead of `next_slot`.
4. Items 5 and 6: owner-indexed methods, memoized `needs_drop`, `std.map` in `prog`.
   These also speed items 2, 4, and 9.
5. Item 4: reverse-order liveness worklist; hoist `stmt_acc` and scratch; per-point
   held-and-live loan sets.
6. Items 7 to 10: `shallow` by reference, hashed `find_fn`/`cg_lay`/`find_str`.
7. Items 11 and 12: cache the Windows toolchain probe and skip `vcvars64.bat`;
   `memcpy` object assembly.
8. Item 15 when the above is done.

Each step must keep `tests/check.sh` and `tests/mir.sh` byte-equal with stage 0 and the
bootstrap hashes equal (`scripts/self.ps1 --stage`); re-measure `llrs0 check
self/llrs.llr` and `llrs0 build self/llrs.llr` after each and record the numbers here.

## How the numbers were taken

```
cargo build --release                                   # stage 0, 2m 12s cold
$env:LLRST_STD = "$PWD\std"; $env:LLRST_RUNTIME = "$PWD\runtime"; $env:SOURCE_DATE_EPOCH = '0'
Measure-Command { llrc  build self\llrs.llr -o target\self\work\llrs0 --std std }
Measure-Command { llrs0 check self\llrs.llr --std std }
Measure-Command { llrs0 build self\llrs.llr -o target\self\work\llrs_t --std std --emit-obj }
Measure-Command { llrs0 build self\llrs.llr -o target\self\work\llrs1 --std std }
Measure-Command { llrs1 build self\llrs.llr -o target\self\work\llrs2 --std std }
```

`llrs1` and `llrs2` hashed equal. The committed `target/release/llrc.exe` from
2026-11-02 predates the `(set x)` constructor form in `std/set.llr` and fails to parse
`std`; a rebuild from `compiler/` is required before measuring.
