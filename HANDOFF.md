# HANDOFF

For the next agent picking up Llrst. Read this, then `docs/canon/README.md`, then the
workflow you need in `docs/workflows/`. Everything below is verifiable from the repo.

## 1. Status

```
commit|ce284b9|tests: lifetimes, struct escape, move through deref
suite|tests/run.sh|done|fails=0
lines|compiler=6018 rs|std=1685 llr|tools=464 llr|max_file=449|budget=500 soft, 1000 hard
plan_todos|11/11 complete|follow_up|pub enforcement done (strict, per module)|panic goldens + idx.oob fields done|flow-sensitive borrow holders done|div.zero is a P| line via sys.checked_div|--checked traps overflow; wadd/wmul always wrap|fs stat via runtime/llr_stat.c|W for unreachable blocks and unused locals|private struct fields
```

The bootstrap plan (spec, compiler, std, hello world, tools, install, docs) is finished.
Nothing is half-built. Open work is in section 7 and section 8.

## 2. Original request and locked decisions

Request (condensed): a compiled language `Llrst` (`.llr`), Rust bootstrap, Rust-like
memory safety, designed for model readability and few tokens, easy to refactor, large
growing std, one right way per task (documented), cross-platform, machine-readable
errors, low context use. Then: list agents' most used tools, reimplement them in Llrst,
install them, document them, write agent workflows. Files under 1000 lines (refactor
near that to under 500). Order of work: reuse, refactor, new code.

Decisions taken with the user before building:

| decision | choice | why |
|----------|--------|-----|
| backend | Cranelift 0.136 (`cranelift-codegen/frontend/module/object/native`) | pure Rust, four hardware ISAs |
| syntax | S-expressions, delimiters `( ) "` only | trivial grammar, no precedence |
| memory | ownership + borrows with NLL-style regions on MIR | Rust model, checked, no GC |
| tools | read grep find edit fmt test diff | highest-frequency agent tools |

## 3. Repo map

| path | role | start here |
|------|------|-----------|
| `compiler/src/` | `llrc` (Rust) | `main.rs` (CLI), `driver.rs` (pipeline) |
| `std/` | standard library, 17 modules, all Llrst | `sys.llr` (externs, panic), `str.llr`, `vec.llr` |
| `tools/` | agent tools, one `.llr` per binary | `llr-read.llr` (smallest), `llr-fmt.llr` + `fmtlint.llr` |
| `tests/` | golden tests and runner | `run.sh`, `run/*.llr+.out`, `err/*.llr+.err` |
| `scripts/` | `install.sh PREFIX`, `check-lines.sh` | |
| `docs/spec/` | grammar, tokens, types, memory, errors | `errors.md` is the code catalogue |
| `docs/canon/` | the one right way per topic | `README.md` |
| `docs/std/` | per-module API tables (generated from signatures) | `README.md` |
| `docs/tools/` | ranked tool list, per-tool output grammars | `README.md` |
| `docs/workflows/` | step lists for agents | `debug.md` maps every error code to an action |

## 4. Pipeline

```mermaid
flowchart LR
  src[".llr"] --> sexpr["sexpr.rs lex+read"] --> parse["parse/ AST"] --> resolve["resolve.rs Program"]
  resolve --> check["check/ THIR per instance"] --> mir["mir/lower* MIR"] --> borrow["borrow/ NLL-lite"]
  borrow --> cg["codegen/ Cranelift"] --> obj[".o"] --> link["link.rs cc"]
```

| pass | file(s) | key types |
|------|---------|-----------|
| read | `sexpr.rs` | `Sexpr::{Atom,Str(Vec<u8>),List}` with `Span` |
| parse | `parse/mod.rs` items, `parse/expr.rs`, `parse/ty.rs` (generics `<T 'a>`, targs) | `ast::Item`, `ast::Expr`, `ast::TyExpr` |
| resolve | `resolve.rs` | `Program { defs: Vec<Def>, modules, methods: HashMap<DefId, HashMap<name, DefId>>, prelude }`; `Loader` loads main, all of `std/`, and `proj.*` modules |
| types | `ty.rs` | `Ty::{Int,F32,F64,Bool,Unit,Never,Ref,Ptr,Adt(DefId,args),Fn,Param,Infer}` |
| check | `check/mod.rs` (unify, scopes, `check_fn(def, targs)`), `expr.rs`, `call.rs`, `pat.rs`, `zonk.rs` | `thir::{FnBody, Expr, ExprKind, Callee, Pat}` |
| MIR | `mir/mod.rs` types + `needs_drop`, `lower.rs`, `lower_call.rs`, `lower_match.rs`, `print.rs` | `Body { locals, blocks }`, `Place { local, proj }`, `Operand::{Copy,Move,Const}`, `Rvalue`, `Stmt::{Assign,Drop,Forget,StorageDead}`, `Term` |
| borrowck | `borrow/mod.rs` (accesses, overlap), `liveness.rs`, `moves.rs`, `loans.rs`, `holders.rs`, `check.rs` | `Loan { place, mutable, two_phase, point, span }`, `Holders` (per-point `(loan, local)` bitsets) |
| codegen | `codegen/mod.rs` (module, symbols, data, ABI), `layout.rs`, `func.rs`, `ops.rs`, `term.rs`, `drop.rs` | `Instance = (DefId, Vec<Ty>)`, `Val::{Scalar,Agg(addr),Unit}` |
| driver | `driver.rs` | checks all non-generic user fns, then mono worklist from `main` (or tests) via `cg.discovered` |

Generics are monomorphized by re-checking the body per instantiation (`check_fn` with
concrete `targs`); there is no polymorphic checking. Only reachable instances are
compiled; std bodies are checked lazily.

## 5. Build, run, verify

```
cargo build                                   # debug llrc
cargo build --release
cargo metadata --format-version 1 | ...target_directory   # sandbox may redirect via CARGO_TARGET_DIR
LLRC=<path to llrc> tests/run.sh              # goldens + every std test + line check
<llrc> run tests/run/hello.llr --std std
<llrc> build std/re.llr --test --std std -o /tmp/t && /tmp/t [FILTER]
<llrc> check tests/err/X.llr --std std 2> tests/err/X.err   # regenerate a golden after a deliberate change
<llrc> build tests/panic/X.llr --std std -o /tmp/p && /tmp/p 2> tests/panic/X.err   # same for panic goldens (exit 101)
<llrc> emit-mir FILE --std std                # MIR dump, for borrow errors
<llrc> build f.llr --target aarch64-unknown-linux-gnu --emit-obj   # also riscv64gc-, s390x-, aarch64-apple-darwin
scripts/install.sh /tmp/prefix                # release build, std copy, 7 tools, hello verify
scripts/check-lines.sh                        # W at 500 lines, fail at 1000
```

std discovery order in `driver::find_std`: `--std`, `$LLRST_STD`, `<exe>/../std`,
`<exe>/../../std`, `<exe>/../share/llrst/std`, `./std`.

`llrc` accepts flags before or after the subcommand, so `LLRC="llrc --std ./std"` works
for `llr-test`.

## 6. Implementation notes (things you would otherwise rediscover)

ABI and data (`codegen/mod.rs`, `codegen/func.rs`)
- Scalars by value; aggregates (structs, enums, `str`, `vec`) passed as a pointer and
  copied into the callee's stack slot in the prologue; aggregate returns via an sret
  pointer as first param. Externs must have scalar params/returns.
- Every MIR local is a stack slot; values are loaded/stored, no SSA variables.
- `"literal"` has type `(& str)` and points at a static 24-byte `{ptr,len,cap=0}` record
  (`str_data`); `str.drop` frees only when `cap > 0`. Length bytes follow target
  endianness (`big_endian`).
- Panic messages are built in std: `str.oob(len, idx)` / `str.oob_range(len, lo, hi)`;
  the compiler only supplies `loc`. Goldens for them live in `tests/panic/`.
- Integer `/` and `%` lower to `sys.checked_div` (`mir/lower_div.rs`): operands widen to
  `i64`, the call carries a `file|line|col` literal, the result truncates back. The
  function's own division stays raw (`Lower::raw_div`) so the rewrite does not recurse.
  Float division is unchanged. `checked_div` always rejects signed `MIN / -1` and `MIN % -1`.
  Goldens: `tests/panic/div_zero*`, `tests/panic/div_min`, `tests/panic/rem_min`, `tests/run/div_ok`.
- `fs.mtime` and `fs.size` call `llr_stat_mtime` / `llr_stat_size` in `runtime/llr_stat.c`.
  `fs.list` calls `llr_readdir_name` there for `d_name`.
  `link.rs` appends `llr_stat.o` from `$LLRST_RUNTIME`, `$LLRST_STD/..`,
  `<exe>/../share/llrst/`, or `./runtime/`. `install.sh` and `tests/run.sh` compile it.
- `mir/lint.rs` runs after borrowck. `flow.unreachable` is an unreachable block that
  contains something other than `Drop`/`StorageDead`/`Nop` (the empty block opened after
  `(ret)` is skipped). `name.unused` is a local or parameter that is never an operand,
  borrow, or discriminant; names starting with `_` are skipped. Warnings do not change
  the exit code. Goldens: `tests/warn/`.
- `+ - * neg` wrap by default. `llrc --checked` traps them (`codegen/ovf.rs` calls
  `sys.abort` with `int.overflow`) using Cranelift `sadd_overflow` / `uadd_overflow` and
  the sub/mul pairs. `wadd` `wsub` `wmul` `wneg` always wrap; the map hash uses `wmul`.
- `(sizeof T)` is `Callee::Sizeof`; `(sys.arch)` is a compile-time `(& str)`; `sys.argc`
  / `sys.argv` read globals set by the generated C `main` (`emit_entry`).
- `--test`: each `(test ...)` becomes `llr_test_i`; `emit_test_entry` calls
  `test.run(name, fnptr)` per test, sums failures, calls `test.done(fails)`. `test.run`
  forks (`std/test.llr`), filters by `argv[1]` substring.

Drops (`mir/lower.rs`, `codegen/func.rs`, `codegen/drop.rs`)
- Each local whose type `needs_drop` has an i8 drop flag: set on whole-local assign and
  call dest, cleared on `Move` rooted at the local, `Stmt::Forget`, and after `Drop`.
- `Stmt::Drop(place)`: whole local -> flag-guarded glue call; projection -> unconditional.
- Drop glue per concrete type: user `drop` method first (any `(impl T (fn drop ((self (&mut T))) unit ...))`), then fields / enum payload by tag.
- Scopes (`Scope { drops, all }`) emit `Drop` then `StorageDead` on exit; `ret`/`brk`/`cont`
  emit drops down to the target depth without popping.
- Match by value: binding a non-copy payload moves it (clears the whole flag), so
  wildcard payload fields are dropped explicitly only in that case (`moved_any`). Without
  moves, the whole value drops at scope end. This fixed a double free; keep it.
- Assignment `(set place v)` drops the old value unless the place goes through a raw ptr.
- `mem.replace`, `mem.forget`, `ptr.read/write` are the escape hatches std uses.

Borrow checker (`borrow/`)
- Moves: forward "maybe moved" bitset per local; `Move` of a whole local (no deref) marks,
  whole-local write clears. Moving out of a field/deref is rejected earlier in lowering
  (`move.field`, `move.deref`), except match bindings on a whole local.
- Loans (`loans.rs`): one per `Rvalue::Ref`, indexed in program order. Holders are a
  forward dataflow in `holders.rs` over `(loan, local)` pairs, union at joins, recorded per
  point. Transfer: a whole-local `Assign`/call dest and `StorageDead` kill that local's
  pairs; `Ref{place}` gens its own loan plus every pair of `place.local` (a reference into a
  holder reaches what it holds); `Use`/`Cast`/`Struct`/`Variant` operands flow pairs when
  either side is ref-bearing; calls flow eligible args into the dest; and for each arg typed
  `(&mut X)` with ref-bearing `X` (`mut_ref_sink_args`), the referents of the mutable loans
  it holds become holders of every other ref argument (so `vec.push (&mut hold) (& a)` makes
  `hold` hold the loan on `a`).
- Eligible call args (`loans::eligible_args`): params sharing a named lifetime with the
  return type; else the single ref param; else `self` if a ref; else all ref params.
- Region of a loan = points where any holder is live (backward liveness, `liveness.rs`).
  A loan never conflicts at its own creation point or with accesses rooted at a holder.
- Conflicts (`check.rs`): mutable loan vs any access; shared loan vs write/move/`&mut`.
  `Drop`/`StorageDead` of the borrowed local with a holder live afterwards is
  `borrow.escape` (reported at the borrow span); loans through a deref are exempt.
- Two-phase: `&mut` created directly as a call argument (or an implicit reborrow in
  argument position) acts shared until the call terminator of the same block.
- Implicit reborrow: using a `&mut` place as an operand produces `&mut *place` into a
  temp instead of moving it (`Lower::reborrow`).

Checker conventions (`check/`)
- Bidirectional: `want` flows into literals, `opt.none`, `vec.new`, generic calls.
  Unsuffixed int literals are `Infer(_, int=true)` and default to `i64` in `finish`.
- `coerce`: `&mut T` where `& T` is wanted inserts `(& (@ e))`.
- `if` without else accepts any then-type (value discarded); `and`/`or` take 2+ args.
- `(try e)` works on `res`/`opt`; variant 0 is ok/some, 1 is err/none (std relies on it).
- `(assert_eq a b)` lowers to `test.eq_str/eq_int/eq_bool(loc, got, want)`; `(panic m)` to
  `sys.panic(loc, m)`; `loc` is a `"file|line|col"` literal built by `loc_lit`.
- Callee resolution order: local fn-typed variable, `lookup_fn`, struct ctor, enum variant,
  builtin (`thir::Builtin::from_path`).

Resolver (`resolve.rs`)
- All `std/*.llr` are always loaded; std types are prelude (no `use`), std module fns
  need `(use std.X)`. `proj.X` loads `X.llr` beside the main file.
- A std file compiled directly (its tests) is registered as both `main` and `std.X`.
- `type.fn` resolves methods first, then module-level fns of the module defining the type
  (`vec.cmp_str`). The typed `sort`/`has`/`index` pairs are gone. `vec.index` takes `eq` on each call.
- Visibility: `Def.is_pub`; `Program::visible(from, d)` is `is_pub || same module`,
  checked in `resolve_ty`, the impl owner lookup, and the checker's `require_visible` at
  every fn/ctor/variant/const/pattern resolution. Error `name.private`. Only `pub` std
  types enter the prelude. `(pub ...)` wraps `fn extern struct enum const` and impl
  methods; `(pub (impl ...))` is a parse error. A struct field is private unless
  `(pub (name type))`. `field_of` and struct constructors outside the defining module
  report `name.private_field`. `mem.replace` of the whole value still copies private fields.

Tools (`tools/`, `docs/tools/`)
- All output is `tag|field|...`; exit 0 ok, 1 tool error (`E|<tool>|0|0|0|code|k=v` on
  stderr), 2 negative result. Lists end with `more|...` when truncated.
- `llr-fmt` is idempotent on the whole repo; the repo is formatted with it. `--lint` is
  clean on `std/` and `tools/`. `fmtlint.llr` is a `proj.` module of `llr-fmt`.
- `std/sexp.llr` is a comment-preserving S-expression reader used by `llr-fmt`. `std/ast.llr`
  is the surface grammar parsed from that tree. `llrc` remains the reference checker.

## 7. Known limits and gaps

| area | limit | impact | where |
|------|-------|--------|-------|
| visibility | a variant is private unless `(pub (name type*))`; same-file code may still use it | another module gets `name.private_variant` | `parse/mod.rs`, `check/call.rs` |
| traits | none; the canon is a named `(fn ...)` on each `vec.sort`, `vec.has`, and `map` call (`docs/canon/collections.md`) | a predicate cannot capture a needle | `std/vec.llr`, `std/map.llr` |
| moves | a field of a struct with no `drop` method can be moved; the other fields still drop. Whole locals otherwise; `move.field`/`move.deref` remain for a pointer or a struct that has `drop` | use `(& ...)`, `str.clone`, `mem.replace` for those | `mir/lower.rs as_operand`, `codegen/func.rs` |
| regions | a field write releases loans of that field; a container of refs declared before its referents reports `borrow.escape` at scope end | a write through `&` or `ptr` releases nothing | `borrow/holders.rs` |
| integers | wrap by default; `--checked` traps `+ - * neg`, `shl` `shr` outside `0..bitwidth`, and an integer cast that does not fit; `MIN / -1` and `MIN % -1` always panic `int.overflow`; float casts still saturate or stay unchecked | without `--checked`, a shift count outside `0..bitwidth` is undefined and an integer cast truncates | `codegen/ops.rs`, `codegen/ovf.rs`, `std/sys.llr` `checked_div` |
| lifetimes | string literals are static; the name `'static` is not a region for other types; struct lifetime params are parsed but not related to field loans beyond flow | over-approximate | `borrow/holders.rs`, `ty.rs lifetimes` |
| init | `(let mut x T)` is checked; a use before every path assigns it is `init.uninit`. A field write does not initialize the struct | drop flags stay off until a whole-local assignment | `borrow/moves.rs` |
| runtime | externs come from the OS file merged into `std.sys` (`sys_posix.llr`, or `sys_windows.llr` on Windows). Directory names come from `llr_readdir_name`. macOS stat and readdir use the same C shim but have not been executed here | the Windows file is selected, not linked | `resolve.rs`, `std/fs.llr` `list`, `runtime/llr_stat.c` |
| regex | backtracking, ASCII, no non-greedy, no pattern backreferences; `--ic` lowercases pattern and text | fine for tool-sized input | `std/re.llr` |
| perf | `vec.sort` allocates a scratch buffer; `map` bucket array never shrinks; a diff middle is capped at 4096 LCS cells | a larger middle is one replace hunk | `std/vec.llr`, `std/map.llr`, `std/diff.llr` |
| cross | `tests/cross.sh` checks `--emit-obj` machine type for aarch64, riscv64, s390x, and apple-darwin; a full `--target` link still needs a clang sysroot | link on target instead | `tests/cross.sh`, `link.rs` |
| fmt | `--width N` (default 100); blank-line policy fixed; comments inside a form force multi-line | by design | `tools/llr-fmt.llr` |
| grep | word boundaries are ASCII `[A-Za-z0-9_]` only | `--word` does not split on Unicode | `std/re.llr` `boundary` |
| diagnostics | `--deny CODE` prints that warning as `E` and exits 1; the flag may be repeated | a warning whose code was not denied still exits 0 | `diag.rs`, `main.rs` |
| tests | `tests/run` compares stderr and the exit status only when `NAME.err` or `NAME.code` exists | a non-zero exit is still ok without `.code` | `tests/run.sh` |
| codegen | all locals in memory, `opt_level=speed` only on IR; `(inline (fn ...))` substitutes a body of at most 4 copy-parameter expressions (`str.at`, `vec.get`) | other calls stay calls | `mir/inline.rs` |

## 8. Suggested next steps, in order
1. `llr-ls PATH` lists one directory (`std/fs.llr` `list`). `llr-find` walks a glob.
2. `llr-rm PATH` deletes a file (`std/fs.llr` `remove`).
3. `llr-git status|diff|log` reads the repo. No commit.
4. The Llrst checker does not yet emit the other `type.*` codes (`type.arity`, `type.no_field`, `type.infer`) or `name.unknown` (`std/check.llr`).
5. `(inline (fn ...))` only substitutes a body of at most 4 expressions whose parameters are scalars, references, pointers, or function pointers, and which has no `ret`, `loop`, `while`, `for`, `match`, `brk`, or `cont`. A recursive call stays a real call (`compiler/src/mir/inline.rs`).
6. A `tests/run` program that exits non-zero is still `ok` unless `NAME.code` is present (`tests/run.sh`).
7. `llr-fmt` blank lines stay fixed: one blank line between top-level forms, except consecutive `use` forms and a form after a comment (`tools/llr-fmt.llr`).

8. `tests/cross.sh` checks the object machine type only. It does not link.
9. POSIX `socket`, `bind`, `listen`, `accept`, `send`, `recv`, `close` in `std/sys.llr`, wrapped by `std/net.llr`. One blocking connection.
10. A `poll` loop in `std/net.llr` so one thread can hold several thousand connections. Not one thread per request.
11. A fixed pthread pool in `std/thread.llr`. A worker is a function pointer.
12. HTTP/1.1 request and response in `std/http.llr`.
13. `llr-http` serves files and a route table `(method, path, handler)`. REST is that table. The acceptance check is several thousand concurrent connections.
14. WebSocket as an HTTP upgrade and framed messages on that server.
15. GraphQL as one `POST /graphql` handler: query text in, JSON out. A documented subset, not a schema compiler.
16. SOAP as one POST handler for a single XML envelope and a single XML response.
17. gRPC as HTTP/2 with a length-prefixed frame. Document the subset; no protobuf compiler.
18. MQTT as its own TCP listener in `std/mqtt.llr`: connect, publish, subscribe.
19. The lifetime name `'static` is still not a region for any type other than a string literal (`borrow/holders.rs`).
20. A function pointer still cannot capture a needle. User traits do not exist (`docs/canon/collections.md`).
21. Without `--checked`, a `shl` or `shr` count outside `0..bitwidth` is still undefined (`codegen/ops.rs`).
22. Float-to-int `(as T n)` still saturates under `--checked`. Int-to-float is not checked (`codegen/ops.rs`).
23. A field move through `&` or `ptr`, or out of a struct that has `drop`, is still `move.field` or `move.deref` (`mir/lower.rs`).
24. A write through `&` or `ptr` still releases no loans (`borrow/holders.rs`).
25. A field write does not initialize the struct (`borrow/moves.rs`).
26. The Windows extern file is selected but not linked, and `fork` stays the POSIX symbol (`std/sys_windows.llr`).
27. A diff middle larger than 4096 LCS cells is one replace hunk, not a minimal edit script (`std/diff.llr`).
28. `llr_readdir_name` has been run on Linux only (`runtime/llr_stat.c`).
29. `+` `-` `*` `neg` still wrap without `--checked` (`codegen/ops.rs`).
30. Inlined calls and drop glue are not panic frames (`codegen/func.rs`).
31. A failed `getcwd`, including a path that does not fit in 1 MiB, still returns an empty string (`std/os.llr`).
32. A warning whose code was not denied still exits 0 (`diag.rs`).
33. Public functions and `std` modules are not warned for unused imports or private functions (`mir/lint.rs`).
34. A generic is still typechecked only for the instances the driver compiles (`compiler/src/driver.rs`).
35. A private variant is still usable in the enum's own file (`check/call.rs`).
36. `mem.replace` of a whole struct still copies private fields (`mir/lower_call.rs`).
37. `vec.index` takes `eq` on each call and cannot capture a needle (`std/vec.llr`).
38. `vec.any` is not a method. A predicate cannot capture a needle (`docs/canon/collections.md`).
39. Heads shrink to a power of two, minimum 16, when `len * 8 <= heads` (`std/map.llr`).
40. A closure captures whole bindings. A captured reference must outlive the closure, and `(clos ...)` does not coerce to `(fn ...)`. A closure body cannot contain a closure (`check/clos.rs`).
41. A backreference compares captured bytes. An unset group does not match (`std/re.llr`).
42. `llr-write` replaces the whole file from stdin and does not create parent directories (`tools/llr-write.llr`).

## 9. Conventions to keep

- Files under 500 lines (`scripts/check-lines.sh` runs in `tests/run.sh`); functions
  under 40 lines (`llr-fmt --lint`).
- Every std function has a `(test ...)` form in its file; every error code has a
  `tests/err` case and a row in `docs/spec/errors.md`; every tool has a `docs/tools` page.
- Reuse std, then refactor std, then write new code.
- Before committing: `llr-fmt <changed .llr>`, `llr-fmt --lint std/*.llr tools/*.llr`,
  `tests/run.sh`. Commit per milestone with a one-line message.
- No prose in diagnostics or tool output; one record per line, `|` separated.
