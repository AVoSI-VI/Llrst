# Error format

Every diagnostic is exactly one line. No prose, no colors, no source excerpts, no carets.
Fields are `|` separated. Field values never contain `|` or newline (escaped as `\x7c`, `\n`).

## Compile-time

```
E|FILE|LINE|COL|LEN|CODE|k=v|k=v...
W|FILE|LINE|COL|LEN|CODE|k=v...
```

| field | meaning |
|-------|---------|
| `E`/`W` | error or warning |
| `FILE` | path as given on the command line |
| `LINE` `COL` | 1-based |
| `LEN` | length of the offending span in bytes |
| `CODE` | stable dotted code, see below |
| `k=v` | code-specific facts; `fix=` is a suggested replacement form when known |

Example:

```
E|src/main.llr|3|5|9|type.mismatch|want=i64|got=(& str)|fix=(str.to_int x)
E|src/main.llr|7|3|1|move.used|name=v|moved_at=6:9
```

Compilation stops after the first pass that produced any `E`. Output goes to stderr.
Exit code 1 on any `E`, 0 otherwise. `--deny CODE` may be repeated. Each named warning is
printed as `E` and the process exits 1. A warning whose code was not denied stays a `W`
and does not fail the build.

| warning | meaning |
|---------|---------|
| `flow.unreachable` | a MIR block nothing jumps to, and it holds real statements (not only drops) |
| `name.unused` | a local or parameter is never read; `name=`. A user generic body is linted even when nothing instantiates it |
| `use.unused` | a `(use ...)` alias is never looked up; `name=`. Not emitted for an `std` module |
| `fn.unused` | a private function is never called; `name=`. Not emitted for `pub`, `extern`, `main`, or an `std` module |

## Runtime panics

```
P|FILE|LINE|COL|CODE|k=v...
```
Exit code 101. `FILE|LINE|COL` is the expression that detected the fault. Later `P|name`
lines are live callers, innermost first. `sys.abort` and `sys.panic` are omitted. An
inlined call pushes a frame named with `fn_path` (`vec.get`, `str.at`) for the substituted
body. Drop glue pushes `drop.<type>` (`drop.box`, `drop.(vec i64)`).

| code | fields | source |
|------|--------|--------|
| `idx.oob` | `len= idx=` (index) or `len= lo= hi=` (slice) | `vec.get` `vec.get_mut` `vec.set` `vec.remove` `vec.swap` `vec.insert` `str.at` `str.slice` |
| `opt.none` | | `opt.unwrap` |
| `res.err` / `res.ok` | | `res.unwrap` / `res.unwrap_err` |
| `div.zero` | | integer `/` and `%`, every width; floats stay IEEE and do not panic |
| `int.overflow` | | `--checked` only for `+` `-` `*` `neg` and for `shl` `shr` with a count outside `0..bitwidth`; signed `MIN / -1` and `MIN % -1` always |
| `cast.overflow` | | `--checked` only: `(as T n)` when an integer does not fit, a truncated float does not fit (including `NaN` and infinities), or an integer is not exact as a float |
| `oom` | | `sys.alloc` `sys.grow` |
| `cli.no_such_opt` / `cli.no_such_flag` | | `cli.get` `cli.has` with an undeclared name |
| user `(panic "code|k=v")` | as written | any program |

`(assert ...)` failing outside a test prints `F|FILE|LINE|COL|assert` (or `assert_eq|want=|got=`)
and also exits 101. Signed `MIN / -1` and `MIN % -1` always panic `int.overflow`.

## Code catalogue

| prefix | emitted by |
|--------|-----------|
| `lex.*` | `lex.unterminated_string` `lex.bad_escape` `lex.bad_suffix` (`got=` on stage 0) |
| `parse.*` | `parse.unbalanced` (`got=` or `want=`) `parse.expect` (`want=` `got=`) `parse.arity` (`form= want= got=`) `parse.bad_head` (`got=` when a known head is rejected; omitted when the head is not an atom) |
| `name.*` | `name.unknown` (`name=`) `name.dup` `name.not_module` (`name=`) `name.not_type` (`name=`) `name.private` (`name= module= fix=`) `name.private_field` (`field= ty= module= fix=`) `name.private_variant` (`variant= ty= module= fix=(pub (name type*))`) |
| `type.*` | `type.mismatch` (`want=` `got=`) `type.arity` (`want=` `got=`) `type.not_callable` (`got=`) `type.no_field` (`ty=` `field=`) `type.no_variant` (`ty=` `variant=`) `type.infer` (`name=` `got=` or `fix=`) `type.not_numeric` (`got=` or `op=` `got=`) `type.not_bool` (`got=`) `type.generic_arity` (`name=` `want=` `got=`) `type.generic_fn_value` (`name=`) `type.unit_bind` (`name=`) `type.not_ref` (`got=`) `type.not_place` `type.bad_cast` (`from=` `to=` on stage 0) `type.try` (`got=` `ret=` on stage 0) `type.assert_eq` (`got=` on stage 0) `type.bad` |
| `lit.*` | `lit.overflow` (`ty=` `got=` on stage 0) |
| `match.*` | `match.missing` (`missing=` or `fix=`) `match.bad_lit` `match.not_enum` (`got=` on stage 0) |
| `move.*` `borrow.*` `mut.*` `init.*` | see memory.md. `mut.required` may be `name=` `fix=` or, on stage 0 only, `through=&` |
| `flow.*` | `flow.brk_outside` `flow.unreachable` (W) |
| `inline.reject` | `why=extern` or `why=form\|got=clos` |
| `clos.nested` | a closure body contains a closure |
| `trait.*` | `trait.missing` (`trait=` `ty=`) `trait.dup` (`trait=` `ty=`) `trait.object` (`why=self`, `why=ref`, or `why=dyn`) `trait.method` (`name=`) |
| `link.*` | `link.failed` (`cmd=` `got=`). The two compilers do not print the same `cmd=` text, so there is no shared `tests/err` golden |
| `io.*` | `io.read` (`path=`; `fix=set LLRST_STD` when `std` is missing; `fix=set LLRST_UI` when `ui` is missing; `use=` when a `proj.` or `ui.` file is missing) `io.write` (`path=` `got=`) |
| `cli.*` | `cli.unknown_flag` (`flag=`) |
| `target.*` | `target.unsupported` (`got=`) |

Adding a code requires adding a row here and a golden test in `tests/err/`.

## Tool output

Tools (`llr-grep` etc.) use the same discipline: one record per line, `|` separated, a
single-letter record tag first. See `docs/tools/`.
