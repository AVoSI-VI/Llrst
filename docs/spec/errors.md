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
Exit code 1 on any `E`, 0 otherwise. A `W` line does not fail the build.

| warning | meaning |
|---------|---------|
| `flow.unreachable` | a MIR block nothing jumps to, and it holds real statements (not only drops) |
| `name.unused` | a local or parameter is never read; `name=` |

## Runtime panics

```
P|FILE|LINE|COL|CODE|k=v...
```
Exit code 101. `FILE|LINE|COL` is the frame that detected the fault, usually inside `std`
(there are no stack traces; find the caller with `llr-grep` on the method name).
`div.zero` is the exception: its location is the `/` or `%` expression itself.

| code | fields | source |
|------|--------|--------|
| `idx.oob` | `len= idx=` (index) or `len= lo= hi=` (slice) | `vec.get` `vec.get_mut` `vec.set` `vec.remove` `vec.swap` `vec.insert` `str.at` `str.slice` |
| `opt.none` | | `opt.unwrap` |
| `res.err` / `res.ok` | | `res.unwrap` / `res.unwrap_err` |
| `div.zero` | | integer `/` and `%`, every width; floats stay IEEE and do not panic |
| `int.overflow` | | `--checked` only: `+` `-` `*` `neg`, and signed `MIN / -1`, `MIN % -1` |
| `oom` | | `sys.alloc` `sys.grow` |
| `cli.no_such_opt` / `cli.no_such_flag` | | `cli.get` `cli.has` with an undeclared name |
| user `(panic "code|k=v")` | as written | any program |

`(assert ...)` failing outside a test prints `F|FILE|LINE|COL|assert` (or `assert_eq|want=|got=`)
and also exits 101. `cast.overflow` is reserved, not emitted. Without `--checked`,
dividing the minimum signed value by `-1` is still a hardware trap. Shifts are not checked.

## Code catalogue

| prefix | emitted by |
|--------|-----------|
| `lex.*` | `lex.unterminated_string` `lex.bad_escape` `lex.bad_char` |
| `parse.*` | `parse.unbalanced` `parse.expect` (`want=`) `parse.arity` (`form= want= got=`) `parse.bad_head` |
| `name.*` | `name.unknown` (`name=`) `name.dup` `name.not_module` `name.not_type` `name.not_fn` `name.private` (`name= module= fix=`) `name.private_field` (`field= ty= module= fix=`) |
| `type.*` | `type.mismatch` `type.arity` `type.not_callable` `type.no_field` `type.no_variant` `type.infer` `type.not_numeric` `type.not_bool` `type.generic_arity` |
| `move.*` `borrow.*` `mut.*` `init.*` | see memory.md |
| `flow.*` | `flow.brk_outside` `flow.missing_ret` `flow.unreachable` (W) |
| `link.*` | `link.failed` (`cmd= status=`) |
| `io.*` | `io.read` (`path=`) |

Adding a code requires adding a row here and a golden test in `tests/err/`.

## Tool output

Tools (`llr-grep` etc.) use the same discipline: one record per line, `|` separated, a
single-letter record tag first. See `docs/tools/`.
