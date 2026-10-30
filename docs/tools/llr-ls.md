# llr-ls

A directory tree, sorted by relative path. `.` and `..` are omitted. Dot names are included
and directories are walked. The path named on the command line is not itself a record.

```
llr-ls PATH [--max N]
```

`--max` defaults to 200.

Output grammar:

```
dir|REL
file|REL
more|N             ; N further paths not shown
```

`|` and newline inside a path are escaped (`\x7c`, `\n`). An empty directory prints
nothing and exits 0.

Errors (stderr, exit 1):

```
E|<ls>|0|0|0|io.list|path=...
```

A missing `PATH` is the `cli` usage error. Canon invocation: `llr-ls PATH`.
