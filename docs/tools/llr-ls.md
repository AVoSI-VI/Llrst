# llr-ls

Names in one directory, sorted. `.` and `..` are omitted. It does not recurse.

```
llr-ls PATH [--max N]
```

`--max` defaults to 200.

Output grammar:

```
NAME
more|N             ; N further names not shown
```

`|` and newline inside a name are escaped (`\x7c`, `\n`). An empty directory prints
nothing and exits 0.

Errors (stderr, exit 1):

```
E|<ls>|0|0|0|io.list|path=...
```

A missing `PATH` is the `cli` usage error. Canon invocation: `llr-ls PATH`.
