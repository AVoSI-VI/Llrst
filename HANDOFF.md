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
powershell -File tests/run.ps1                # same suite on Windows
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
  `link.rs` appends `llr_stat.o` on Unix, or compiles and links `llr_win.c` and `llr_wsa.c` on Windows,
  from `$LLRST_RUNTIME`, `$LLRST_STD/..`, `<exe>/../share/llrst/`, or `./runtime/`.
  `install.sh` and `tests/run.sh` compile the Unix object. `tests/run.ps1` does not;
  `llrc` compiles the Windows object.
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
| traits | `Eq` `Ord` `Hash` `Clone` in `std/cmp.llr`; `(& (dyn Trait))` is data plus vtable | a trait method is not generic; an impl type is a concrete name; a method that returns `Self` is not a trait object | `compiler/src/check/traits.rs`, `std/cmp.llr` |
| moves | a field of a struct with no `drop` method can be moved; the other fields still drop. Whole locals otherwise; `move.field`/`move.deref` remain for a pointer or a struct that has `drop` | use `(& ...)`, `str.clone`, `mem.replace` for those | `mir/lower.rs as_operand`, `codegen/func.rs` |
| regions | a field write releases loans of that field; a container of refs declared before its referents reports `borrow.escape` at scope end | a write through `&` or `ptr` releases nothing | `borrow/holders.rs` |
| integers | wrap by default; `--checked` traps `+ - * neg`, `shl` `shr` outside `0..bitwidth`, and an integer cast that does not fit; `MIN / -1` and `MIN % -1` always panic `int.overflow`; float casts still saturate or stay unchecked | without `--checked`, a shift count is masked with `bitwidth - 1` and an integer cast truncates | `codegen/ops.rs`, `codegen/ovf.rs`, `std/sys.llr` `checked_div` |
| lifetimes | `'static` is a region for any type; a `'static` reference comes from a literal, a const, or another `'static` reference | any other named lifetime is only the cross-function flow in `eligible_args` | `borrow/holders.rs`, `borrow/loans.rs` |
| init | `(let mut x T)` is checked; a use before every path assigns it is `init.uninit`. A field write does not initialize the struct | drop flags stay off until a whole-local assignment | `borrow/moves.rs` |
| runtime | externs come from the OS file merged into `std.sys` (`sys_posix.llr`, or `sys_windows.llr` on Windows). Directory names come from `llr_readdir_name`. macOS stat and readdir use the same C shim but have not been executed here | `fork` does not copy an address space; Windows clients run on a thread and each std test is re-execed | `resolve.rs`, `std/fs.llr` `list`, `runtime/llr_stat.c`, `runtime/llr_win.c` |
| net | `poll_serve` holds up to 4096 fds on one thread; the callback is a function pointer plus a context pointer | `connect` is 127.0.0.1; the Windows shim translates the 8-byte pollfd and errno 11 and 4 | `std/net.llr`, `runtime/llr_win.c` |
| thread | a pool of `n` pthreads created at `start`; a job is `(fn ((ptr u8)) unit)` plus a context pointer | the count does not grow; a worker cannot capture a binding; the Windows shim maps the pthread names onto Win32 | `std/thread.llr`, `runtime/llr_win.c` |
| http | `llr-http` serves files, exact routes, and a WebSocket text echo on one poll loop | a route path is exact; chunked bodies are rejected; a WebSocket payload must be one text frame under 126 bytes; at most 4095 clients | `std/http.llr`, `tools/llr-http.llr` |
| gql | `POST /graphql` takes the query text and returns JSON for `hello`, `ping`, and `add` | no schema, no variables, and no nested selection sets | `std/gql.llr` |
| xml | one element document of elements and text | no attributes, namespaces, comments, or declarations; entities are `&lt;` `&gt;` `&amp;` | `std/xml.llr` |
| soap | `POST /soap` takes one XML envelope and returns XML for `hello`, `ping`, and `add` | a body is one operation | `std/soap.llr` |
| grpc | one HTTP/2 unary call; the message is length-prefixed text for `/hello`, `/ping`, and `/add` | no Huffman, no dynamic table, no protobuf; one stream and one message | `std/grpc.llr` |
| mqtt | one connection speaks connect, subscribe, and QoS 0 publish | exact topics, one-byte remaining length, no wildcards or auth | `std/mqtt.llr` |
| regex | backtracking, ASCII, no non-greedy, no pattern backreferences; `--ic` lowercases pattern and text | fine for tool-sized input | `std/re.llr` |
| perf | `vec.sort` allocates a scratch buffer; `map` bucket array never shrinks; a diff middle is capped at 4096 LCS cells | a larger middle is one replace hunk | `std/vec.llr`, `std/map.llr`, `std/diff.llr` |
| cross | `tests/cross.sh` links aarch64, riscv64, and s390x freestanding with lld; apple-darwin stays an object check | a full `--target` link still needs a clang sysroot | `tests/cross.sh` |
| fmt | `--width N` (default 100); `--blank N` (default 1) between top-level forms; a comment inside a form forces multi-line | consecutive `use` forms and a form after a comment get no blank line | `tools/llr-fmt.llr` |
| grep | word boundaries are ASCII `[A-Za-z0-9_]` only | `--word` does not split on Unicode | `std/re.llr` `boundary` |
| diagnostics | `--deny CODE` prints that warning as `E` and exits 1; the flag may be repeated | a warning whose code was not denied still exits 0 | `diag.rs`, `main.rs` |
| tests | `tests/run` compares stderr only when `NAME.err` exists. Exit status is 0 unless `NAME.code` is present | stderr is still ignored without `.err` | `tests/run.sh` |
| codegen | all locals in memory, `opt_level=speed` only on IR; `(inline (fn ...))` substitutes the body, including `ret`, loops, `match`, and owned parameters (`str.at`, `vec.get`) | a recursive call stays a call; a closure in the body is `inline.reject` | `mir/inline.rs` |

## 8. Suggested next steps, in order
1. Float-to-int `(as T n)` still saturates under `--checked`. Int-to-float is not checked (`codegen/ops.rs`).
2. A field move through `&` or `ptr`, or out of a struct that has `drop`, is still `move.field` or `move.deref` (`mir/lower.rs`).
3. A write through `&` or `ptr` still releases no loans (`borrow/holders.rs`).
4. A field write does not initialize the struct (`borrow/moves.rs`).
5. `fork` still cannot resume a Cranelift stack. Windows runs a client on a thread via `llr_call_child` and re-execs each std test (`std/test.llr`, `runtime/llr_win.c`).
6. A diff middle larger than 4096 LCS cells is one replace hunk, not a minimal edit script (`std/diff.llr`).
7. `llr_readdir_name` reads `dirent` on Unix and `WIN32_FIND_DATA` on Windows (`runtime/llr_stat.c`, `runtime/llr_win.c`).
8. `+` `-` `*` `neg` still wrap without `--checked` (`codegen/ops.rs`).
9. Inlined calls and drop glue are not panic frames (`codegen/func.rs`).
10. A failed `getcwd`, including a path that does not fit in 1 MiB, still returns an empty string (`std/os.llr`).
11. A warning whose code was not denied still exits 0 (`diag.rs`).
12. Public functions and `std` modules are not warned for unused imports or private functions (`mir/lint.rs`).
13. A generic is still typechecked only for the instances the driver compiles (`compiler/src/driver.rs`).
14. A private variant is still usable in the enum's own file (`check/call.rs`).
15. `mem.replace` of a whole struct still copies private fields (`mir/lower_call.rs`).
16. `vec.index` takes `eq` on each call and cannot capture a needle (`std/vec.llr`).
17. `vec.any` is not a method. A predicate cannot capture a needle (`docs/canon/collections.md`).
18. Heads shrink to a power of two, minimum 16, when `len * 8 <= heads` (`std/map.llr`).
19. A closure captures whole bindings. A captured reference must outlive the closure, and `(clos ...)` does not coerce to `(fn ...)`. A closure body cannot contain a closure (`check/clos.rs`).
20. A backreference compares captured bytes. An unset group does not match (`std/re.llr`).
21. `llr-write` replaces the whole file from stdin and does not create parent directories (`tools/llr-write.llr`).
22. `llr-ls` lists one directory and does not recurse (`tools/llr-ls.llr`).
23. `llr-rm` deletes a file and does not remove a directory (`tools/llr-rm.llr`).
24. `llr-git` reads status, diff, and log and does not change the repo (`tools/llr-git.llr`).
25. `llr-check` still does not emit `type.not_callable`, `type.no_variant`, `type.not_numeric`, `type.not_bool`, or `type.generic_arity` (`std/check.llr`).
26. A recursive `(inline ...)` call stays a real call, and a closure in an inline body is still `inline.reject|why=form|got=clos` (`compiler/src/mir/inline.rs`).
27. Stderr of a `tests/run` program is still ignored unless `NAME.err` exists (`tests/run.sh`).
28. Consecutive `use` forms and a form after a comment still get no blank line, and a comment inside a form still forces a line break (`tools/llr-fmt.llr`).
29. A full `llrc build --target` link still needs a clang sysroot, and apple-darwin is not linked (`tests/cross.sh`, `compiler/src/link.rs`).
30. `net.connect` is 127.0.0.1 only (`std/net.llr`).
31. The poll table holds 4096 fds and the ready callback cannot capture a binding (`std/net.llr`).
32. The pool size is fixed at start and a worker cannot capture a binding (`std/thread.llr`).
33. Chunked transfer and trailers are not parsed (`std/http.llr`).
34. A route path is exact, chunked bodies are still rejected, and the poll table holds at most 4095 clients (`std/http.llr`).
35. A WebSocket payload must be a single text frame under 126 bytes (`std/http.llr`).
36. There is no GraphQL schema, no variables, and no nested selection sets (`std/gql.llr`).
37. A SOAP body is one operation, and XML has no attributes, namespaces, or comments (`std/soap.llr`, `std/xml.llr`).
38. There is no Huffman coding, no HPACK dynamic table, no protobuf, and only one stream and one message (`std/grpc.llr`).
39. The MQTT broker is one connection, QoS 0, exact topics, and a one-byte remaining length (`std/mqtt.llr`).
40. A function pointer still cannot capture a needle. A trait method is not generic, an impl type is a concrete name, and a method that returns `Self` is not a trait object (`compiler/src/check/traits.rs`).

## 9. Conventions to keep

- Files under 500 lines (`scripts/check-lines.sh` runs in `tests/run.sh`); functions
  under 40 lines (`llr-fmt --lint`).
- Every std function has a `(test ...)` form in its file; every error code has a
  `tests/err` case and a row in `docs/spec/errors.md`; every tool has a `docs/tools` page.
- Reuse std, then refactor std, then write new code.
- Before committing: `llr-fmt <changed .llr>`, `llr-fmt --lint std/*.llr tools/*.llr`,
  `tests/run.sh`. Commit per milestone with a one-line message.
- No prose in diagnostics or tool output; one record per line, `|` separated.
