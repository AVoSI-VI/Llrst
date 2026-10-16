# llr-edit

Exact string replacement with a uniqueness check.

```
llr-edit FILE OLD NEW [--all] [--files] [--regex]
```

| flag | meaning |
|------|---------|
| `--all` | replace every occurrence (default requires exactly one) |
| `--files` | OLD and NEW are paths to files containing the text (multi-line edits) |
| `--regex` | OLD is a `std.re` pattern. NEW expands `$$` to `$`, `$0` to the whole match, `$N` to capture N |

Output grammar:

```
ok|FILE|replaced=N
```

Errors (stderr, exit 1):

```
E|<edit>|0|0|0|edit.not_found|old=...
E|<edit>|0|0|0|edit.ambiguous|count=N|fix=--all or a longer OLD
E|<edit>|0|0|0|edit.empty_old|fix=give non-empty OLD
E|<edit>|0|0|0|io.read|path=...
E|<edit>|0|0|0|re.parse|at=N|what=...
E|<edit>|0|0|0|re.expand|group=N
```

Canon: read the lines first (`llr-read`), copy OLD verbatim including indentation, make
OLD long enough to be unique, then `llr-fmt FILE` and `llr-test`.
