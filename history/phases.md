# Phases 0-7 (done)

This was the **Phases** section of `HANDOFF.md` until 2026-11-03. Every phase is closed.
The live handoff is [../HANDOFF.md](../HANDOFF.md). The text below is kept as written.

Status legend: `[ ]` not started, `[~]` in progress, `[x]` done.

## Phases

Every row of the former "Known limits" table and every item of the former "Suggested next
steps" list is either a decision above or one item below. Items marked *verify* come from
code reading and were not run.

### Phase 0: handoff and repo hygiene (no behaviour change) `[x]`

Files: `README.md`, `docs/tools/README.md`, `.gitignore`, `scripts/check-lines.sh`, `tests/run.ps1`, `runtime/llr_win.c`, `self/mono.llr`, `self/subst.llr`.

1. `[x]` `README.md`: the compiler is stage-2 `llrs` from `self/`, not "Rust on Cranelift"; add `self/`, `runtime/`, `scripts/` rows; the tools row lacks `llr-doc` and `llr-check`; say that `cargo build --release` produces stage 0 only.
2. `[x]` `docs/tools/README.md`: the `llr-http` row has three cells in a four-column table.
3. `[x]` `poll_probe.obj` is tracked at the repo root with no reference in the tree. `runtime/llr_win.obj` and `runtime/llr_wsa.obj` are tracked while `scripts/install.ps1` lines 76-82 rewrite them on every install; `.gitignore` has `*.o` but no `*.obj`. Removing tracked files needs the owner's confirmation.
4. `[x]` `scripts/check-lines.sh` and the inline copy in `tests/run.ps1` (lines 271-287): add `self/` and `runtime/*.c`; keep the two in sync or make `run.ps1` call one script.
5. `[x]` Split `runtime/llr_win.c` (1024 lines) under 500 per file; `install.ps1` compiles each piece.
6. `[x]` `self/mono.llr` and `self/subst.llr` are imported by nothing and build on `std.check`; retire them or wire them, then decide whether `std/check*.llr` stays only to back `llr-check`. Retired both files. `std/check*.llr` stays only to back `llr-check`.
7. `[x]` `runtime/llr_win.c` is LF in the index and CRLF in a Windows checkout; add `.gitattributes` or state the rule.
8. `[x]` Commit: `docs: handoff phases, line check covers self and runtime`.

### Phase 1: self/ parity with stage 0 `[x]`

Files: `self/drive.llr`, `self/check.llr`, `self/llrs.llr`, `self/lint.llr`, `self/inline.llr`, `self/collect.llr`, `self/resolve.llr`, `self/thir_clos.llr`, `self/cg_clos.llr`, `self/link.llr`, `self/link_win.llr`, `std/ast_expr.llr`, `std/os.llr`.

1. `[x]` `(test ...)` bodies run `borrow.run` and `lint.body`. Generic instances and std functions are MIR-linted when compiled; `lint.generics` walks only never-instantiated generics.
2. `[x]` `llrs emit-mir` keeps diagnostics, prints the dump, and exits 1 on error. `tests/mir.sh` and `tests/mir.ps1` compare stderr too.
3. `[x]` `llrs run` builds `llr_run_N` under `TMPDIR`, `TEMP`, `TMP`, or `/tmp`, and removes `link.host_exe` of that path.
4. `[x]` `os.shell` returns the `system()` code unshifted when `sys.os_id` is 2.
5. `[x]` Span-less errors use the main file and `1|1|0`. Collect errors and `io.read` are full records. Parse errors carry `len=`; arity sites emit `parse.arity` and a non-atom head emits `parse.bad_head`. `link.failed` puts the full command in `cmd=`.
6. `[x]` A failed inline re-check is appended to the lowerer's diagnostics.
7. `[x]` Generic `let`s use a scope. `fn.unused` resolves `(T.m ...)` and value uses through `lookup_fn`. Under `--test`, `lint.items` scans `p.tests`.
8. `[x]` Trait impls report `trait.method` and `type.mismatch`. Bounds stay on `prog.def` and `trait.missing` is reported at the call. Goldens: `tests/err/trait_method`, `tests/err/trait_bound`. `std/check_more.llr` teaches `llr-check` those two.
9. `[x]` Closure capture binds `for` vars, match bindings, and inner params. The trampoline classifies GPR, XMM, and stack args. `tests/run/clos.llr` covers a float capture and a seven-int call.
10. `[x]` Recorded under Decisions. Both compilers typecheck a generic only for compiled instances.
11. `[x]` No std directory is `io.read|path=std|fix=set LLRST_STD` on the main file at `1|1|0`.
12. `[x]` Commit: `self: test bodies are borrow-checked, emit-mir reports, lint and trait parity`.

### Phase 2: language and borrow checker `[x]`

Files: `self/lower.llr`, `self/lower_call.llr`, `self/holders.llr`, `self/moves.llr`, `self/thir_call.llr`, `self/thir_clos.llr`, `self/collect.llr`, and the stage 0 twins `compiler/src/mir/lower.rs`, `borrow/holders.rs`, `borrow/moves.rs`, `check/call.rs`, `check/clos.rs`, `check/traits.rs`.

1. `[x]` Moves: a field of a struct with no `drop` method can be moved and the other fields still drop; a field move through `&` or `ptr`, or out of a struct that has `drop`, is still `move.field` or `move.deref` (`lower.llr` `as_operand`). A later use of that field is `move.used`.
2. `[x]` Regions: a field write releases loans of that field; a write through `&` or `ptr` releases nothing; a container of refs declared before its referents reports `borrow.escape` at scope end (`holders.llr`). Field notes join across blocks.
3. `[x]` Init: `(let mut x T)` is checked and a use before every path assigns it is `init.uninit`; a field write does not initialize the struct, so drop flags stay off until a whole-local assignment (`moves.llr`). A drop of one field is flag-guarded.
4. `[x]` Lifetimes: `'static` is a region for any type and comes from a literal, a const, or another `'static` reference (`static_ck.llr`); any other named lifetime is only the cross-function flow in `eligible_args` (`loans.llr`). A struct field copied into `& 'static` is static only when that field was stored from one of those sources.
5. `[x]` Visibility: a private variant is still usable in the enum's own file (`thir_call.llr`, `check/call.rs`); `mem.replace` of a whole struct still copies private fields (`lower_call.llr`). A constructor in another file still rejects a private field.
6. `[x]` Closures: a closure captures whole bindings; a captured reference must outlive the closure; `(clos ...)` does not coerce to `(fn ...)`; a closure body cannot contain a closure (`thir_clos.llr`, `check/clos.rs`).
7. `[x]` Traits: `Eq Ord Hash Clone` live in `std/cmp.llr`; `(& (dyn Trait))` is data plus vtable. A trait method is not generic, an impl type is a concrete name, a method that returns `Self` is not a trait object, and a function pointer cannot capture a needle (`collect.llr`, `thir_call.llr`, `check/traits.rs`).
8. `[x]` Commit: `lang: field moves through refs, writes through refs release loans, field init`.

### Phase 3: diagnostics, catalogue, goldens `[x]`

Files: `docs/spec/errors.md`, `docs/spec/memory.md`, `docs/spec/types.md`, `docs/spec/tokens.md`, `docs/workflows/debug.md`, `tests/err/`, `tests/panic/`, `tests/run.sh`, `tests/run.ps1`, `self/diag.llr`, `self/lint.llr`, `self/inline.llr`, `self/cg_frame.llr`.

1. `[x]` `--deny CODE` prints that warning as `E` and exits 1 and may be repeated; a warning whose code was not denied still exits 0 (`diag.llr`, `compiler/src/diag.rs`). Warnings do not fail a build except through `--deny`. Goldens: `tests/warn/deny_unused.code`, `tests/warn/deny_two`.
2. `[x]` Public functions and `std` modules are not warned for unused imports or private functions (`lint.llr`, `mir/lint.rs`). That exemption stays. Goldens: `tests/warn/pub_kept`, `tests/warn/std_silent`.
3. `[x]` Inlined calls push a panic frame named with `fn_path`. Drop glue pushes `drop.<type>` (`cg_frame.llr`, `codegen/trace.rs`, `cg_drop.llr`). `tests/panic/str_oob`, `vec_oob`, and `drop_frame` lock it.
4. `[x]` `(inline (fn ...))` substitutes the body, including `ret`, loops, `match`, and owned parameters; a recursive call stays a call and a closure in the body is `inline.reject|why=form|got=clos` (`inline.llr`). All locals are in memory; `opt_level=speed` applies to stage 0 IR only. Documented in `docs/spec/grammar.md` and `docs/install.md`.
5. `[x]` A run test with no `NAME.err` fails when stderr is non-empty. Exit status is 0 unless `NAME.code` is present (`tests/run.sh`, `tests/run.ps1`). `tests/run/status.code` is 3.
6. `[x]` `docs/spec/errors.md` lists the emitted codes, including `move.field` and `move.deref` in `memory.md`. `link.failed` is `cmd=` `got=`. Dead rows `lex.bad_char`, `name.not_fn`, and `flow.missing_ret` are gone. Signed `MIN / -1` always panics `int.overflow`.
7. `[x]` `tests/err` covers the missing check codes except `link.failed`, whose `cmd=` text differs between the two linkers. `tests/panic` covers `res.ok`, `oom`, `cli.no_such_opt`, and `cli.no_such_flag`. `llr-check` is not required to match `name.dup`, `name.not_module`, `name.not_type`, `flow.brk_outside`, or `io.read` until Phase 4.
8. `[x]` `docs/workflows/debug.md` has a row for each code that was missing, and describes inline and drop frames.
9. `[x]` `docs/spec/tokens.md` lists the missing keywords and `'static`. `types.md` documents `never`, trait-bound `map` keys, `mem.replace`, `mem.forget`, `sys.argc` / `argv` / `arch`, and `(sizeof T)`. `memory.md` rule 3 matches the copy list. `docs/install.md` documents the `llrc` commands.
10. `[x]` Commit: `diag: catalogue matches emitters, goldens for every code`.

### Phase 4: std `[x]`

Files: `std/*.llr`, `docs/std/*.md`, `docs/std/README.md`, `docs/canon/io.md`, `docs/canon/README.md`, `docs/workflows/refactor.md`, `docs/workflows/std.md`, `std/check.llr`, `tools/fmtlint.llr`.

1. `[x]` A diff middle larger than 4096 LCS cells is one replace hunk, not a minimal edit script (`std/diff.llr`). `diff.ops` returns `(vec op)` while `op` is private, so a caller cannot name the result.
2. `[x]` A failed `getcwd`, including a path that does not fit in 1 MiB, returns an empty string (`std/os.llr`).
3. `[x]` `vec.index` cannot capture a needle; `vec.any` is not a method and a predicate cannot capture a needle (`std/vec.llr`, `docs/canon/collections.md`). Depends on Phase 2 item 6.
4. `[x]` Zero `(test ...)` forms: `std/io.llr`, `std/test.llr`, `std/cmp.llr`, `std/ast_expr.llr`, `std/check_expr.llr`. Untested functions elsewhere, largest first: `std/check.llr` 43 of 45, `std/http.llr` 35 of 47, `std/ast.llr` 24 of 25, `std/re.llr` 23 of 30, `std/str.llr` 22 of 36, `std/grpc.llr` 18 of 23, `std/vec.llr` 17 of 26, `std/net.llr` 14 of 24, `std/mqtt.llr` 14 of 19, `std/sexp.llr` 14 of 19, `std/sha.llr` 14 of 17, `std/xml.llr` 12 of 20, `std/gql.llr` 10 of 13, `std/thread.llr` 10 of 14, `std/diff.llr` 9 of 12, `std/map.llr` 7 of 14, `std/sys.llr` 6 of 7, `std/soap.llr` 6 of 9. Counts are name references inside a `(test` body in the same file; indirect coverage is not counted. `docs/std/README.md` says "every pub fn" where the rule above says every function; pick one.
5. `[x]` Regenerate `docs/std/` with `llr-doc`: `vec.md` and `map.md` still show fn-pointer params and helpers (`vec.cmp_str`, `map.hash_str`) that no longer exist; `http.md`, `os.md`, `thread.md` lack `## Types`; `sys.md` shows only the POSIX externs and `sys.os_id` is undocumented; `cmp.md` has a hand-written `## Traits` that `llr-doc` would clobber. Add `docs/std/sys_posix.md`, `docs/std/sys_windows.md`, and the `ast_expr`, `sys_posix`, `sys_windows` rows to `docs/std/README.md` (32 rows, 35 modules); its `re` row lacks `\b` and `\N`.
6. `[x]` `llr-check` (`std/check.llr` `std_dir`) has no `--std` flag and none of `llrc`'s fallbacks, so the installed tool fails outside the repo unless `LLRST_STD` is set; it prints the first error only and skips `name.dup`, `name.not_module`, `name.not_type`, `flow.brk_outside`, `match.*`, `type.unit_bind`, `type.try`; load failures are returned as the diagnostic body instead of `E|<check>|0|0|0|...`.
7. `[x]` `docs/canon/README.md` promises a lint code for every `no` form; `tools/fmtlint.llr` lacks `canon.bind.redundant_type`, `canon.bind.type_on_let`, `canon.program.use_one`. `docs/canon/io.md` has no blessed form for `fs.size append remove mkdir rename`, `os.shell capture arch is_tty`, `io.put eput`, `path.stem is_abs clean`.
8. `[x]` `docs/workflows/refactor.md` says to edit `docs/std/<mod>.md` by hand; `std.md` step 6 regenerates with `llr-doc`. Keep one. `std.md` names only `tests/run.sh` and says std types are visible everywhere (only `pub` ones are).
9. `[x]` Commit: `std: tests for every function, regenerated docs, llr-check finds std`.

### Phase 5: tools `[x]`

Files: `tools/llr-write.llr`, `tools/llr-ls.llr`, `tools/llr-rm.llr`, `tools/llr-fmt.llr`, `tools/llr-http.llr`, `tools/llr-diff.llr`, `tools/llr-test.llr`, `docs/tools/*.md`, `docs/workflows/new-tool.md`, `docs/workflows/README.md`.

1. `[x]` `llr-write` replaces the whole file from stdin and does not create parent directories.
2. `[x]` `llr-ls` lists one directory and does not recurse.
3. `[x]` `llr-rm` deletes a file and does not remove a directory.
4. `[x]` `llr-fmt`: `--width N` (default 100), `--blank N` (default 1) between top-level forms; consecutive `use` forms and a form after a comment get no blank line; a comment inside a form forces multi-line.
5. `[x]` `llr-http` serves a file as `200` with a suffix type (`.html` `text/html`, `.wasm` `application/wasm`, `.js` `text/javascript`, otherwise `text/plain`). `docs/tools/llr-http.md` lists `/health /graphql /soap /ws`.
6. `[x]` `docs/tools/README.md` says every list tool accepts `--max N` and every record starts with a tag: `llr-diff` and `llr-test` have no `--max`; `llr-read`, `llr-find`, `llr-grep`, `llr-ls` records have no tag; `more|` payloads differ per tool (`more|N`, `more|limit reached`, `more|NEXT|TOTAL`); `docs/workflows/README.md` tag list omits `usage file status log`. `llr-test.md` describes a forked child only (Windows re-execs). `fmtlint.llr` is named only here, not in `llr-fmt.md`.
7. `[x]` `docs/workflows/new-tool.md` step 7 adds a row to `docs/tools/README.md` but not to `README.md`; step 8 names `install.sh` only.
8. `[x]` Commit: `tools: write makes directories, ls recurses, rm removes directories`.

### Phase 6: platform, link, targets, scripts `[x]`

Files: `self/drive.llr`, `self/load.llr`, `self/link.llr`, `self/link_win.llr`, `self/llrs.llr`, `self/cg_mod.llr`, `std/test.llr`, `runtime/llr_win.c`, `runtime/llr_win_sock.c`, `runtime/llr_win_proc.c`, `runtime/llr_stat.c`, `scripts/install.sh`, `scripts/install.ps1`, `scripts/self.sh`, `tests/run.sh`, `tests/run.ps1`, `tests/cross.sh`, `docs/install.md`.

1. `[x]` Runtime: externs come from the OS file merged into `std.sys` (`sys_posix.llr` or `sys_windows.llr`). `fork` does not copy a Cranelift or `self/` stack, so Windows runs a client on a thread via `llr_call_child` and re-execs each std test (`std/test.llr`, `runtime/llr_win_proc.c`). On Darwin `open fcntl bind connect socket setrlimit` are imported as `llr_*` from `runtime/llr_stat.c`.
2. `[x]` The installed compiler accepts `--target` when it is empty, an `x86_64` triple without `apple`, or a triple containing `aarch64` or `arm64` (`drive.llr` `target_ok`, `docs/spec/targets.md`). x86-64 is ELF or COFF. aarch64 is ELF (`e_machine` 183) or Mach-O arm64 on a Darwin triple. riscv64 and s390x stay on stage 0. `tests/cross.sh` links the three ELF targets freestanding with lld for stage 0, skips riscv64 and s390x for a `self/` compiler, and checks apple-darwin with `file`. `self/` has no endianness switch (`cg_mod.llr` `put_u64`).
3. `[x]` `--target` does not switch `sys_windows.llr` / `sys_posix.llr`; `load.llr` follows the host. The `x86_64-pc-windows-gnu` os_id check in `tests/cross.sh` needs stage 0.
4. `[x]` Unix link is host `cc` with `-pie -pthread` (`link.llr`), not `clang --target`; a Darwin host never sets the darwin renames and Apple clang rejects `-pie`.
5. `[x]` Windows link is `cl` only (`link_win.llr`); the installed compiler does not compile `llr_wsa.c`, `llr_win.c`, `llr_win_sock.c`, or `llr_win_proc.c` and does not probe clang or gcc. On Unix it never rebuilds `llr_stat.o` either (`docs/install.md`).
6. `[x]` `llrc run` quotes arguments with POSIX single quotes (`llrs.llr` `quote`), so `cmd` does not receive the vector stage 0 passes.
7. `[x]` Scripts: no `scripts/self.ps1`, so the Windows bootstrap runs only inside `install.ps1`; `install.ps1` has no `SOURCE_DATE_EPOCH` (relies on `/Brepro`); the `os_id` expected-output hook exists only in `tests/run.ps1` lines 96-99; neither runner invokes `tests/mir`, `tests/check`, `tests/cross`, or `tests/self`; no installer installs `examples/hello.llr` (both compile a temp hello). `docs/install.md` says "`tests/run.sh` is the check for stage 0" while the suite is also the check for the installed compiler.
8. `[x]` Commit: `platform: one bootstrap script per host, runners call the differentials`.

### Phase 7: net, thread, and protocol subsets `[x]`

Files: `std/net.llr`, `std/thread.llr`, `std/http.llr`, `std/gql.llr`, `std/xml.llr`, `std/soap.llr`, `std/grpc.llr`, `std/mqtt.llr`, `tools/llr-http.llr`, `runtime/llr_win_sock.c`, `runtime/llr_win_proc.c`.

1. `[x]` `net.connect` takes a dotted IPv4. `poll_serve` does not cap the client table. The ready callback is a closure. `runtime/llr_win_sock.c` `poll` follows the caller's count up to `FD_MAX`. `map_wsa` maps the socket errno values `net` can see.
2. `[x]` `thread`: `start` creates `n` workers and `submit` grows the pool by one when the queue is longer. A job is `(clos () unit)`. `CreateThread` uses an 8 MiB stack.
3. `[x]` `http`: chunked bodies and trailers are decoded. A route path may be exact, end in `*`, or use `{name}`. WebSocket messages reassemble continuation frames at every length form. The client table is not capped. Body parsing is `http_body.llr`. Frames are `http_ws.llr`.
4. `[x]` `gql`: a schema, variables, nested selections, aliases, and fragments. `POST /graphql` accepts a JSON body.
5. `[x]` `xml` reads attributes, namespaces, comments, CDATA, a declaration, and the predefined entities. `soap` is a SOAP 1.1 envelope with a header and Fault.
6. `[x]` `grpc`: HPACK with Huffman and a dynamic table, many unary streams, and protobuf messages for `/hello /ping /add`.
7. `[x]` `mqtt`: a 3.1.1 broker. Variable remaining length, `+` and `#`, username and password, and QoS 0, 1, and 2.
8. `[x]` Commit: `net: chunked http, growing pool, protocol subsets widened`.

## Progress log

- 2026-10-15: bootstrap plan and its eleven follow-ups done; see `history/bootstrap.md`.
- 2026-10-26: self-hosting roadmap done, `llrs2` installed as `llrc` (`54ed8fe`, merged `ebaed76`); see `history/self-hosting.md`.
- 2026-10-27: this file reshaped into rules, decisions, architecture, line budget, and phases 0-7. Every former limit row and next-step item was placed once. Gaps found by reading `self/`, `std/`, `tools/`, `tests/`, `scripts/`, and `docs/` were added as `[ ]` items; items marked *verify* were not executed. No code changed. The suite was not re-run.
- 2026-10-27: Phase 0. README names Llrst 0.1.0 and stage-2 `llrs`. The line check covers `self/` and `runtime/*.c`. `llr_win.c` is split into `llr_win.c`, `llr_win_sock.c`, and `llr_win_proc.c`. Tracked `.obj` files are gone. `mono.llr` and `subst.llr` are retired; `std/check*.llr` stays for `llr-check`. `.gitattributes` stores `*.c` and `*.h` as LF. `tests/run.ps1` fails=3, all Cranelift emit panics on float casts (`run/cast_float`, `panic/cast_f_high`, `panic/cast_f_neg`). Shim tests, including `std/fs`, `std/net`, and `std/thread`, passed.
- 2026-10-27: Phase 1. Test bodies and compiled instances are borrow-checked and MIR-linted. `emit-mir` keeps diagnostics and exits 1. `run` uses a temp executable. `os.shell` is unshifted on Windows. Diagnostics are full records. Inline re-check keeps its error. Lint scopes generic lets and counts method and value uses. Trait impls are shape-checked and bounds fail at the call. Closures bind loop and match names and pass floats and stack arguments. A missing std directory is `io.read`. `tests/run.ps1` fails=3, the same Cranelift float-cast panics. `tests/check.ps1` and `tests/mir.ps1` match stage 0.
- 2026-10-27: Phase 2. A moved field is `move.used` in `self/` as well as stage 0. Field-loan notes join across blocks. A field write does not set a drop flag; a single-field drop is flag-guarded. A struct field is `'static` only when its stored reference is. Private-field constructors are rejected outside the defining file. `(clos ...)` does not coerce to `(fn ...)`, and a generic trait method is `parse.expect|want=(types)`. `tests/check.ps1` and `tests/mir.ps1` match stage 0. `tests/run.ps1` fails=3, the same Cranelift float-cast panics.
- 2026-10-28: Phase 3. `--deny` may be repeated and is the only way a warning fails the build. `pub` functions and `std` modules stay exempt from `use.unused` and `fn.unused`. Inlined calls and drop glue push panic frames (`str.at`, `vec.check`, `drop.box`). The frame list is thread-local (`llr_trace_push`). The catalogue matches the emitters. `link.failed` has no shared golden because the two linkers print different `cmd=` text. `oom` panics on this Windows host. `tests/check.ps1` and `tests/mir.ps1` match stage 0. `tests/run.ps1` fails=3, the same Cranelift float-cast panics.
- 2026-10-30: Phase 4. A large diff middle is a linear-space LCS and `op` is public. `os.cwd` returns `(res str str)` and `io.cwd` on failure. `vec.any` and `vec.find` take a closure that can capture a needle. `llr-doc` emits traits, constants, and generic clauses; `docs/std` was regenerated, including `sys_posix`, `sys_windows`, `ast_expr`, and `check_more`. `llr-check` takes `--std`, searches the same places as `llrc`, and reports `name.dup`, `name.not_module`, `name.not_type`, `flow.brk_outside`, `match.bad_lit`, `match.not_enum`, `match.missing`, and `type.try`. The rule is every function, not every pub fn. `tests/run.ps1` fails=3, the same Cranelift float-cast panics.
- 2026-10-30: Phase 5. `llr-write` creates parent directories. `llr-ls` lists a directory tree as `dir|` and `file|`. `llr-rm` deletes a file or a directory tree. `llr-fmt` already had `--width`, `--blank`, and the comment rules. List records are `line|`, `path|`, `hit|`, `dir|`, and `file|`, and a cut list is `more|N`. `llr-diff` and `llr-test` take `--max` (default 0, no limit). `llr-http` documents `GET /health`, `POST /graphql`, and `POST /soap`; files stay `200 text/plain`, and a websocket upgrade is any GET with those headers. `llr-test` documents the Windows re-exec. `fmtlint.llr` is named in `llr-fmt.md`. `tests/run.ps1` fails=3, the same Cranelift float-cast panics.
- 2026-10-30: Phase 6. `--target` selects `sys_windows.llr` or `sys_posix.llr`. Integer bytes follow a big-endian flag that is set for an `s390x` triple; accepted targets stay little-endian x86-64, and the other ISAs stay on stage 0. A Unix triple links with `clang --target`. An empty triple uses `cc`, and a Darwin host omits `-pie` and renames the six libc calls. Windows probes `clang`, `gcc`, `cc`, then `cl`, and links the prebuilt shim extension that exists. `llrc run` passes arguments through `llr_exec`. `scripts/self.ps1` is the Windows bootstrap. Installers copy `examples/` and verify with `examples/hello.llr`. `tests/run.sh` expects `os_id` 2 when `OS=Windows_NT`. Both runners call check, mir, self, and cross against stage 0. Goldens and std tests fails=0 under the rebuilt self compiler, including the three former Cranelift float-cast cases. `tests/check.ps1` and `tests/mir.ps1` fails=0 against stage 0. `tests/self.ps1` fails=33 because `llr-check` does not emit borrow errors or `--deny`. On this host `tests/cross.sh` is `fail|setup|lld=missing`.
- 2026-10-30: Darwin phase 0. Cast emitters moved to `self/cg_cast.llr`; `cg_op.eval_cast` calls it. `tests/x64_ref.sh` snapshots x86-64 ELF and COFF. `tests/run` and `std` objects stayed byte-equal. The `self/llrs` object grew by the forwarder; each moved function body is the same bytes, and that snapshot was refreshed. `tests/check.sh` and `tests/mir.sh` fails=0.
- 2026-10-30: Darwin. Stage 0 is the installed `llrc`: `scripts/self.sh` stops at `llrs1` with `ld: unknown file type` because `self/` emits ELF. Goldens, std tests, `tests/check.sh`, and `tests/mir.sh` fails=0. `tests/self.sh` fails=33 because `llr-check` does not emit borrow errors or `--deny`. `tests/cross.sh` links aarch64; riscv64 and s390x need a clang with those backends (Homebrew `llvm` is keg-only at `/opt/homebrew/opt/llvm/bin/clang`). The plan to self-host on this Mac is `DARWIN.md`.
- 2026-10-31: Darwin phase 5. `self/` accepts an empty triple, `x86_64` without `apple`, and `aarch64` or `arm64`, including Mach-O on a Darwin triple and ELF for `aarch64-unknown-linux-gnu`. riscv64 and s390x stay on stage 0. `scripts/install.sh` installs `llrs2` on Darwin. `tests/check.sh` and `tests/mir.sh` fails=0. `tests/x64_ref.sh` is byte-equal after refreshing the `self/llrs` snapshots. `LLRC=llrs0 tests/cross.sh` fails=0.
- 2026-10-31: Phase 7. `connect` takes a dotted IPv4. Poll and thread callbacks are closures, and both pools grow. HTTP decodes chunked bodies and reassembles WebSocket messages. GraphQL runs a schema with variables and nested selections. XML reads attributes and namespaces. SOAP 1.1, unary gRPC with HPACK and protobuf, and an MQTT 3.1.1 broker are in std. Windows `poll` is not clamped at 4096. `LLRC=llrc tests/run.sh`: goldens, std tests, lines, `tests/check.sh`, `tests/mir.sh`, and `tests/cross.sh` fails=0. `tests/self.sh` stops at `llrs0` with `use.unused|name=opt` in `self/drive.llr`, and stage 0 still cannot build `llr-check` (`type.mismatch` in `std/check.llr`).
- 2026-10-31: A trait in another module is `module.Trait`, and `(&mut box)` passed into a call is the fat pointer (`tests/run/box_mut`, `tests/run/dyn_qual`). `ui/` phase 4 is in `UIFRAME.md`.
- 2026-10-31: wasm32 (`UIFRAME.md` phase 6). `self/wasm_*.llr` lower MIR to one Wasm module for `wasm32-wasi` and `wasm32-unknown-unknown`; `load.llr` appends `std/sys_wasm.llr` and `sys.os_id` is 3; `std/alloc.llr` is the allocator. Stage 0 `resolve.rs` skips `sys_wasm` like the other host files so def ids stay equal. `self/mir_print.llr` prints `&mut box`, and a `(& box)` through an inferred type takes the resolved trait id. `tests/wasm.sh` fails=0 under wasmtime 50; goldens, std tests, `tests/check.sh`, and `tests/mir.sh` fails=0 with the rebuilt self compiler. `ui-mod/event` fails `type.arity` on the nested `(opt.some (event.pointer ...))` pattern in `self/` (stage 0 accepts it); that predates this phase and is fixed on 2026-11-01.
- 2026-11-01: Two blockers. `std/gql_schema.llr` declared `pub` types `ty` and `field`, and `std/gql_parse.llr` declared `doc`, repeating `std/ast.llr` and `std/xml.llr`; stage 0's prelude map kept the last one while `self/` and `std/check.llr` kept the first, so stage 0 could not build `llr-check` or `llrs0`. They are `gtype`, `gfield`, and `gdoc`. A repeated `pub` std type name is now `name.dup|name=|module=` in all three checkers (`compiler/src/prelude.rs`, `self/collect.llr`, `std/check_load.llr`), and the first keeps the name. `self/thir_pat.llr` copied pattern children without their own children, so any variant pattern nested in another had zero sub-patterns; it borrows them now (`tests/run/match_nested`). `scripts/self.sh --stage` reaches `stage2=equal`.
- 2026-11-01: `llr-check` is a front end over `llrc check` (`os.exec`, `os.exe_path`): `$LLRC`, then `llrc` beside the tool, then `PATH`; arguments and the exit code pass through. The second checker in `std/check*.llr` is retired, so the two can no longer disagree. `tests/run.sh` and `tests/self.sh` export `LLRC` and compare `llr-check` on every `tests/err` and `tests/warn` golden, including borrow, move, init, warnings, and `--deny`. `tests/self.sh` fails=0.
- 2026-11-02: `llr-ui` phase 13. `state` gained `save`, `load`, and `layout`. `--emit-dylib` links a shared library whose `llr_ui_boot` keeps the element and render arenas across a reload. A `-1` layout, a changed layout, or a different tree restarts through that `save` string. The browser page does the same with `sessionStorage` when the wasm heap base moves. `llr-ui dump`, `shot`, and `run` are in `docs/tools/llr-ui.md`. The plan is `history/ui.md` and the workflow is `docs/workflows/ui.md`.
- 2026-11-03: Llrst is 0.4.5. `llrc version` prints `llrc|0.4.5` from stage 0 and `self/`.
- 2026-11-03: moved here from `HANDOFF.md`.
