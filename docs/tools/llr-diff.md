# llr-diff

Line diff of two files.

```
llr-diff A B [--context N]
```

Output grammar (lines are 1-based; `=` carries both line numbers):

```
stat|removed=R|added=A
=|I|J|text
-|I|text
+|J|text
```

`--context N` (default 2) keeps N equal lines around each change; `-1` prints everything. A middle of more than 4096 LCS cells is one replace hunk, not a minimal edit.

Exit: 0 identical, 2 different, 1 unreadable input.
