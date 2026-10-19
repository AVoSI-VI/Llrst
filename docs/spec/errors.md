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
Exit code 1 on any `E`, 0 otherwise. `--deny CODE` prints that warning as `E` and exits 1.
A warning whose code was not denied stays a `W` and does not fail the build.

| warning | meaning |
|---------|---------|
| `flow.unreachable` | a MIR block nothing jumps to, and it holds real statements (not only drops) |
| `name.unused` | a local or parameter is never read; `name=`. A user generic body is linted even when nothing instantiates it |
| `use.unused` | a `(use ...)` alias is never looked up; `name=` |
| `fn.unused` | a private function is never called; `name=` |

## Runtime panics

```
P|FILE|LINE|COL|CODE|k=v...
```
Exit code 101. `FILE|LINE|COL` is the expression that detected the fault. Later `P|name`
lines are live callers, innermost first. `sys.abort` and `sys.panic` are omitted. An
inlined body does not get its own frame.

| code | fields | source |
|------|--------|--------|
| `idx.oob` | `len= idx=` (index) or `len= lo= hi=` (slice) | `vec.get` `vec.get_mut` `vec.set` `vec.remove` `vec.swap` `vec.insert` `str.at` `str.slice` |
| `opt.none` | | `opt.unwrap` |
| `res.err` / `res.ok` | | `res.unwrap` / `res.unwrap_err` |
| `div.zero` | | integer `/` and `%`, every width; floats stay IEEE and do not panic |
| `int.overflow` | | `--checked` only for `+` `-` `*` `neg` and for `shl` `shr` with a count outside `0..bitwidth`; signed `MIN / -1` and `MIN % -1` always |
| `cast.overflow` | | `--checked` only: integer `(as T n)` when `n` does not fit in `T` |
| `oom` | | `sys.alloc` `sys.grow` |
| `cli.no_such_opt` / `cli.no_such_flag` | | `cli.get` `cli.has` with an undeclared name |
| user `(panic "code|k=v")` | as written | any program |

`(assert ...)` failing outside a test prints `F|FILE|LINE|COL|assert` (or `assert_eq|want=|got=`)
and also exits 101. Without `--checked`, dividing the minimum signed value by `-1` is still a hardware trap.

## Code catalogue

| prefix | emitted by |
|--------|-----------|
| `lex.*` | `lex.unterminated_string` `lex.bad_escape` `lex.bad_char` |
| `parse.*` | `parse.unbalanced` `parse.expect` (`want=`) `parse.arity` (`form= want= got=`) `parse.bad_head` |
| `name.*` | `name.unknown` (`name=`) `name.dup` `name.not_module` `name.not_type` `name.not_fn` `name.private` (`name= module= fix=`) `name.private_field` (`field= ty= module= fix=`) `name.private_variant` (`variant= ty= module= fix=(pub (name type*))`) |
| `type.*` | `type.mismatch` `type.arity` `type.not_callable` `type.no_field` `type.no_variant` `type.infer` `type.not_numeric` `type.not_bool` `type.generic_arity` |
| `move.*` `borrow.*` `mut.*` `init.*` | see memory.md |
| `flow.*` | `flow.brk_outside` `flow.missing_ret` `flow.unreachable` (W) |
| `inline.reject` | `why=extern` or `why=large\|limit=4\|got=` or `why=form\|got=` or `why=param\|name=` |
| `clos.nested` | a closure body contains a closure |
| `link.*` | `link.failed` (`cmd= status=`) |
| `io.*` | `io.read` (`path=`) |

Adding a code requires adding a row here and a golden test in `tests/err/`.

## Tool output

Tools (`llr-grep` etc.) use the same discipline: one record per line, `|` separated, a
single-letter record tag first. See `docs/tools/`.
