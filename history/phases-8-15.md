# Phases 8-15 (done)

This was the **Phases** section of `HANDOFF.md` until 2026-11-08. Every phase here is closed.
The live handoff is [../HANDOFF.md](../HANDOFF.md). Phases 0-7 are [phases.md](phases.md).
Phases 16-23 stayed in the handoff. The text below is kept as written.

Status legend: `[ ]` not started, `[~]` in progress, `[x]` done.

## Phases

Phases 8 to 15 schedule every row of the 2026-11-03 **Gaps** catalogue.
Order: general-purpose parity (strings, numbers, collections, io, time) first, then the
language changes std needs, then protocols, platform, and tooling. Every new function gets a
`(test ...)`, a `docs/std` row from `llr-doc`, and a canon line when the call shape is new. A
new std module is one file under 500 lines; `str.llr` (257 lines) and `vec.llr` (264) are
split at 400 if the additions take them there.

### Phase 8: handoff and docs hygiene (no behaviour change) `[x]`

Files: `HANDOFF.md`, `docs/std/README.md`, `docs/std/fs.md`, `docs/std/http_type.md`, `docs/spec/tokens.md`, `std/http_mime.llr`, `std/http.llr`.

1. `[x]` `llr-doc --out docs/std std/*.llr`: adds `fs.now_ms` to `fs.md` and creates `http_type.md`. Add the `http_type` row to `docs/std/README.md`.
2. `[x]` `std/http_mime.llr` is a test with no API. Move `http_mime` into `std/http.llr` tests if `http.llr` can stay under 500 after the Phase 9 split of its helpers, else keep the file and add a README row that says it is test-only.
3. `[x]` `docs/spec/tokens.md` documents `0x` integer literals, which both parsers accept.
4. `[x]` Recount the **Line budget** table; drop rows under 400.
5. `[x]` Commit: `docs: std index complete, hex literals documented, line budget recounted`.

### Phase 9: strings and numbers `[x]`

Files: `std/str.llr`, new `std/str_more.llr` if `str.llr` passes 400 (verify first that the resolver allows a second `(impl str ...)` in another std file; else the new functions are module functions `str_more.X`), `std/math.llr`, `std/math_trig.llr`, `std/fmt.llr`, new `std/scan.llr`, `docs/canon/strings.md`, then `self/`, `ui/`, `tools/` callers.

1. `[x]` `str`: `upper trim_start trim_end rfind rfind_byte split_str replace_byte pad_right`; `is_space is_digit is_alpha is_alnum` become `pub`; `from_bytes ((& (vec u8)))` and `to_bytes`; `utf8_at ((& str) i)` returning codepoint and length, `push_utf8 ((&mut str) cp)`, `utf8_ok`; `to_int_radix from_int_radix`; `to_f64` and `from_f64 (x digits)` (shortest-roundtrip is out of scope; fixed digits). `str.to_int` keeps its signature.
2. `[x]` `math`: `imin imax iclamp iabs ipow align_up approx`; export `exp ln`; add `tan asin acos log2 log10 fmod lerp clamp`, each with a `32` twin; one `nan` `inf` `pow2` definition shared by `math.llr` and `math_trig.llr`.
3. `[x]` `fmt`: `hex (n width)`, `float (x digits)`, `pad_right`, and `fields ((& str))` returning `(vec str)` for a pipe record, with `field_int`.
4. `[x]` `std/scan.llr`: a cursor over `(& str)` with `peek at skip_ws is_name read_name hex_dig expect`. `gql_parse gql_schema sexp re xml json_read` use it and drop their copies. `eshort` `hname` `hval` live once in `http.llr` and are `pub` for `http_body` and `http_ws`.
5. `[x]` Delete the duplicates outside std and call the new functions: `nib hex_byte hex` (7 files in `self/`), `fmt_f` and `milli` (`ui/`), `slash` (6), `last_seg` (4), `last_dot` (2), `cstr_eq` (3), `exe_dir` (`self/load.llr:29`, `ui/font.llr:48` use `os.exe_path` and `path.dir`), `ui/font.llr` `dir_of`, `self/cg_drop.llr` and `self/wasm_drop.llr` `cat`. The compiler must stay byte-equal: `tests/x64_ref.sh`, `tests/check.sh`, `tests/mir.sh`.
6. `[x]` Canon rows in `docs/canon/strings.md` for every new form; `docs/std` regenerated.
7. `[x]` Commit: `std: str, math, fmt, and scan cover the helpers self and ui had copied`.

### Phase 10: collections, opt, res `[x]`

Files: `std/vec.llr`, new `std/vec_more.llr` under the Phase 9 split rule, `std/map.llr`, `std/opt.llr`, `std/res.llr`, `std/cmp.llr`, new `std/set.llr`, new `std/deque.llr`, `docs/canon/collections.md`, `docs/canon/errors.md`.

1. `[x]` `vec`: `clone <T:Clone>`, `extend ((& (vec T))) <T:Clone>`, `filled (n x) <T:Clone>`, `resize`, `truncate`, `first`, `swap_remove`, `eq <T:Eq>`, `dedup <T:Eq>`, `min max <T:Ord>`, `sort_by ((clos ((& T) (& T)) i64))` (stable, same merge as `sort`), `retain ((clos ((& T)) bool))`. `vec.clone_str` stays as a forwarder for one release.
2. `[x]` `map`: `values ((& m)) (vec (& V))`, `each ((clos ((& K) (& V)) unit))`, `clear`, `take`, `get_or_insert ((&mut m) k (clos () V)) (&mut V)`; `del` keeps `bool`.
3. `[x]` `std/set.llr`: `(set K)` over `(map K unit)` with `new add has del len items <K:Clone:Ord>`. `tools/deps.llr` `seen` and the parallel-vector lookups in `self/cg_mod.llr:278` and `self/wasm_mod.llr:166` use it where the compiler output stays byte-equal.
4. `[x]` `std/deque.llr`: ring buffer `(deque T)` with `push_back push_front pop_front pop_back len get`.
5. `[x]` `opt`: `map and_then ok_or expect unwrap_or_else take`; `res`: `map map_err ok err expect`. Closure-taking forms follow `vec.any`. Replace the 15 inline `(match x ((opt.some n) n) ((opt.none) d))` sites with `opt.or`.
6. `[x]` `cmp`: `Eq Ord Hash Clone` for `u8 i32 u32 u64 bool`; `Eq Clone` for `f64 f32` (no `Ord`, no `Hash`). `Eq Ord Hash Clone` for `vec opt res` wait for Phase 12 item 1.
7. `[x]` `vec.each map filter` as closure forms, last, and the `docs/canon/collections.md` line "No map or filter in v1" becomes the blessed form. Replace the 20 element-clone loops with `vec.clone`; the compiler stays byte-equal (`tests/x64_ref.sh`).
8. `[x]` Commit: `std: vec clone and sort_by, map iteration, set, deque, opt and res combinators`.

### Phase 11: io, fs, path, os, time, random, bytes, encoding, json, test `[x]`

Files: `std/io.llr`, `std/fs.llr`, `std/path.llr`, `std/os.llr`, new `std/time.llr`, new `std/rand.llr`, new `std/bytes.llr`, new `std/url.llr`, `std/sha.llr`, new `std/sha256.llr`, `std/json.llr`, `std/json_read.llr`, `std/test.llr`, `std/sys_posix.llr`, `std/sys_windows.llr`, `std/sys_wasm.llr`, `runtime/llr_now.c` (or `llr_stat.c`), `runtime/llr_win.c`, `docs/canon/io.md`, `docs/canon/tests.md`.

1. `[x]` `io.read_line (fd) (opt str)` with a per-fd buffer, `io.lines (fd) (vec str)`. `fs.copy is_file temp_dir abs`. `path.sep` is `/` or `\` by `sys.os_id`; `join base dir` accept both on `os_id` 2; `clean` normalizes to `/`.
2. `[x]` `os.set_env home pid`, `os.run (file args stdin) (res cmd str)` where `cmd` gains `err`; `os.capture` is unchanged.
3. `[x]` `std/time.llr`: `now_ms mono_ms sleep_ms utc (ms) date` with `(struct date (pub year month day hour min sec ms))`, `iso8601 parse_iso`. `fs.now_ms` forwards to `time.now_ms` for one release. Shims: `llr_mono_ms`, `llr_sleep_ms`, `llr_entropy` in the host runtime; wasm uses `clock_time_get` and `random_get`.
4. `[x]` `std/rand.llr`: xorshift64\* `(struct rng (s u64))` with `seed next below f64 shuffle <T>`, `rand.entropy () rng`.
5. `[x]` `std/bytes.llr`: `rd_le rd_be (b (& (vec u8)) at n) i64`, `wr_le wr_be ((&mut (vec u8)) v n)`, `uvar svar` and their readers, `str` to and from `(vec u8)` through `str.from_bytes`. `h2 hpack http_ws mqtt_pkt pb sha` use it; `self/elf* coff macho* x64_buf wasm` and `ui/ttf*` follow when byte-equal.
6. `[x]` `sha.b64_decode`, `std/sha256.llr` `sha256`, `bytes.crc32`; `std/url.llr` `encode decode` (RFC 3986 unreserved set), `url.parse` into `(struct url (pub scheme host port path query))`; `str.utf8_ok` from Phase 9 is the validator.
7. `[x]` `json`: `jval` kind 2 carries floats, `json_read` parses `-?d+(.d+)?([eE][+-]?d+)?` and `\u` escapes as UTF-8 including surrogate pairs; `json.show (v (& jval)) str` and `json.pretty (v indent)`; `json.quote` unchanged.
8. `[x]` `test.eq_f64 (loc got want eps)`, `test.ne_int`, `test.ne_str`; `assert_near` and `assert_ne` forms in both compilers lower to them (grammar, `docs/canon/tests.md`, `tests/run/assert_more`).
9. `[x]` Commit: `std: time, rand, bytes, url, sha256, line io, json floats, float asserts`.

### Phase 12: language changes std needs `[x]`

Files: `compiler/src/parse/mod.rs`, `compiler/src/parse/expr.rs`, `compiler/src/resolve.rs`, `compiler/src/check/traits.rs`, `compiler/src/check/expr.rs`, `compiler/src/traitres.rs`, `compiler/src/mir/lower.rs`, `self/collect.llr`, `self/thir_trait.llr`, `self/thir_expr.llr`, `self/lit.llr`, `self/lower.llr`, `std/cmp.llr`, `std/test.llr`, `docs/spec/grammar.md`, `docs/spec/tokens.md`, `docs/spec/types.md`, `docs/spec/errors.md`, `tests/err`, `tests/run`.

1. `[x]` Generic trait impls: `(impl Eq (vec T) <T:Eq> (fn eq ...))`. `is_trait_impl` accepts a list type; the impl is monomorphized per instance like a generic fn; `trait.dup` compares the head type only. Then `Eq Ord Hash Clone` for `(vec T)`, `(opt T)`, `(res T E)` in `std/cmp.llr`, and `vec.clone_str` is deleted.
2. `[x]` `(pub (trait Show (fn show ((& Self)) str)))` in `std/cmp.llr` with impls for every primitive, `str`, `vec`, `opt`, `res`. `assert_eq` and `assert_ne` resolve through `Show` and `Eq` when the operand is not `(& str)`, `i64`, `bool`, or `u8`, so `type.assert_eq` is emitted only for a type with no `Show`. `f64` and `f32` use `Show` plus `test.eq_f64` with an exact compare.
3. `[x]` `bnot` as a unary integer form in both parsers and both backends (`x64_int`, `a64_int`, `wasm_op`).
4. `[x]` Literals: `0b` `0o`, `_` separators, `1e9` and `1.5e-3` floats, `\u{HHHH}` string escapes encoded as UTF-8. `docs/spec/tokens.md` lists all of them.
5. `[x]` New errors: `const.not_lit` for a `const` whose initializer is not a literal (today silent, `compiler/src/mir/lower.rs:293`); struct and enum bounds are honored at instantiation (`trait.missing` when the argument does not implement the bound). Goldens in `tests/err`, `const.not_lit` in `docs/spec/errors.md`.
6. `[x]` `tests/check.sh` and `tests/mir.sh` byte-equal; `scripts/self.sh --stage` reaches `stage2=equal`.
7. `[x]` Commit: `lang: generic trait impls, Show, bnot, numeric literal forms, const.not_lit`.

### Phase 13: concurrency and net `[x]`

Files: `std/thread.llr`, new `std/sync.llr`, `std/net.llr`, new `std/net_dns.llr`, `std/sys_posix.llr`, `std/sys_windows.llr`, `std/sys_wasm.llr`, `runtime/llr_win_sock.c`, `runtime/llr_stat.c`, `docs/canon/io.md`.

1. `[x]` `std/sync.llr`: `(mutex T)` with `new lock ((&mut m) (clos ((&mut T)) unit))` so the guard cannot escape, and `(chan T)` with `new send recv try_recv close` over a mutex and condition variable (`pthread_cond_*` externs on POSIX, `CONDITION_VARIABLE` on Windows). Sending requires nothing beyond ownership; there is no `Send` trait, documented in `docs/spec/memory.md`.
2. `[x]` `thread.spawn ((clos () T)) (handle T)` and `thread.join (handle T) T`; the pool is unchanged.
3. `[x]` `net.resolve (host) (res (vec str) str)` over `getaddrinfo`; `connect` accepts a name or an IPv6 literal; `listen6`; `net.udp_bind udp_send udp_recv`; `net.set_timeout (fd ms)`. Windows maps the new errno values in `map_wsa`.
4. `[x]` wasm: every new extern writes `target.unsupported` and returns `-1` as the existing ones do.
5. `[x]` Commit: `std: mutex, channel, spawn and join, dns, udp, ipv6, socket timeouts`.

### Phase 14: language ergonomics `[x]`

Files: both parsers, both checkers, both lowerers (`compiler/src/parse/expr.rs`, `check/pat.rs`, `mir/lower_match.rs`; `self/thir_pat.llr`, `self/lower_match.llr`, `self/lower_cf.llr`, `self/load.llr`, `self/resolve.llr`), `docs/spec/grammar.md`, `docs/canon/flow.md`, `docs/canon/program.md`, `tests/err`, `tests/run`.

Large. Each item is its own commit with `check.sh` and `mir.sh` byte-equal at the end.

1. `[x]` Patterns: guard `(PAT (if EXPR) EXPR)`, or-pattern `(or PAT PAT...)`, string literal patterns through `str.eq`, struct patterns `(point x y)`. `match.missing` still covers the result.
2. `[x]` `(for NAME lo hi step BODY...)` with a signed step; `(brk LABEL)` and `(cont LABEL)` with `(loop LABEL ...)`; `(while ...)` unchanged. Canon keeps `[lo hi)` as the default.
3. `[x]` `(use proj.a.b)` loads `a/b.llr` under the main file's directory; `(pub (use ...))` re-exports.
4. `[x]` `try` converts the error through `(trait From ...)` when the function's error type implements it; otherwise today's `type.mismatch`.
5. `[x]` Commit per item: `lang: match guards and or-patterns`, `lang: for step and labeled brk`, `lang: nested proj and pub use`, `lang: try through From`.

### Phase 15: compiler and tooling `[x]`

Files: `self/cg_frame.llr`, `self/cg_op.llr`, `self/isa_*.llr`, `self/elf.llr`, `self/elf_out.llr`, `self/macho_out.llr`, `self/coff.llr`, `self/drive.llr`, `tools/llr-doc.llr`, new `tools/llr-bench.llr`, new `tools/llr-lsp.llr`, `docs/tools/`, `docs/spec/targets.md`.

Large; listed last. Each item is its own commit.

1. `[x]` Scalar locals live in callee-saved registers when a function has no address-taken use of them; the ISA seam gains a register slot kind. `tests/x64_ref.sh` snapshots are refreshed on purpose and the suite stays green. [history/comp.md](comp.md) records the 2026-11-06 `llrs1`/`llrs0` build ratio as 1.14 (21.6 s / 19.0 s), about a 14% compile-time lever.
2. `[x]` `--debug` emits `.debug_line` (DWARF 5) in ELF and Mach-O and a line table in COFF, from the statement spans `self/lower.llr` already sets.
3. `[x]` `riscv64` in `self/` as a third ISA behind `cg_isa`; `x86_64-apple-darwin` Mach-O x86-64. `tests/cross.sh` stops skipping them.
4. `[x]` `llr-doc --private` prints private items; `llr-bench FILE` times each `(bench NAME ...)` form and prints `bench|NAME|ns`; `llr-lsp` speaks LSP over stdio and answers diagnostics by running `llrc check`, definition by `llr-grep` on the resolver's `def` records.
5. `[x]` Commit per item: `self: registers for scalar locals`, `self: debug line tables`, `self: riscv64 and x86_64 darwin`, `tools: llr-doc private, llr-bench, llr-lsp`.
