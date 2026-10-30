# llr-find

Files matching a glob, newest first.

```
llr-find PATTERN [DIR] [--max N]
```

Glob syntax: `*` `?` `[a-z]` `[^x]` within a segment, `**` for any directory depth.
A pattern without `/` matches the base name anywhere under DIR.

Output grammar:

```
path|PATH|MTIME   ; MTIME is unix seconds. `|` and newline in PATH are escaped
more|N             ; N further files not shown
```

Exit: 0 found, 2 none.

Canon invocation: `llr-find "*.llr" tools`, `llr-find "std/**/re*.llr" .`
