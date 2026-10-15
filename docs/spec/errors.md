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
Exit code 1 on any `E`, 0 otherwise.

## Runtime panics

```
P|FILE|LINE|COL|CODE|k=v...
```
Exit code 101. Codes: `idx.oob` (`len= idx=`), `opt.none`, `res.err` (`err=`), `div.zero`,
`cast.overflow`, `assert` (`msg=`), `oom`.

## Code catalogue

| prefix | emitted by |
|--------|-----------|
| `lex.*` | `lex.unterminated_string` `lex.bad_escape` `lex.bad_char` |
| `parse.*` | `parse.unbalanced` `parse.expect` (`want=`) `parse.arity` (`form= want= got=`) `parse.bad_head` |
| `name.*` | `name.unknown` (`name=`) `name.dup` `name.not_module` `name.not_type` `name.not_fn` |
| `type.*` | `type.mismatch` `type.arity` `type.not_callable` `type.no_field` `type.no_variant` `type.infer` `type.not_numeric` `type.not_bool` `type.generic_arity` |
| `move.*` `borrow.*` `mut.*` `init.*` | see memory.md |
| `flow.*` | `flow.brk_outside` `flow.missing_ret` `flow.unreachable` (W) |
| `link.*` | `link.failed` (`cmd= status=`) |
| `io.*` | `io.read` (`path=`) |

Adding a code requires adding a row here and a golden test in `tests/err/`.

## Tool output

Tools (`llr-grep` etc.) use the same discipline: one record per line, `|` separated, a
single-letter record tag first. See `docs/tools/`.
