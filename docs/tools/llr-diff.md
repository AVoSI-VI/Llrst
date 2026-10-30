# llr-diff

Line diff of two files.

```
llr-diff A B [--context N] [--max N]
```

Output grammar (lines are 1-based; `=` carries both line numbers):

```
stat|removed=R|added=A
=|I|J|text
-|I|text
+|J|text
more|N
```

`--context N` (default 2) keeps N equal lines around each change; `-1` prints everything. The middle is a minimal edit. `--max N` (default 0, no limit) prints at most N change lines after `stat|`. `more|N` is how many change lines were not shown. Exit 2 still means the files differ, from the full stat.

Exit: 0 identical, 2 different, 1 unreadable input.
