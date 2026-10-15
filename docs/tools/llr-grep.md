# llr-grep

Regex or literal search across files.

```
llr-grep PATTERN [PATH...] [--glob G] [--fixed] [--ic] [--max N]
```

| flag | default | meaning |
|------|---------|---------|
| `PATH...` | `.` | files or directories (directories are walked, dotfiles skipped) |
| `--glob` | `` | only files whose base name (or path when G has `/`) matches G |
| `--fixed` | off | PATTERN is a literal, not a regex |
| `--ic` | off | case-insensitive (ASCII) |
| `--max` | 200 | stop after N matches |

Regex syntax: `std.re`: `. ^ $ * + ? {n,m} [..] [^..] \d \w \s ( ) |`. Parentheses in
Llrst source must be escaped: `"\\(fn main"`.

Output grammar:

```
PATH|LINE|COL|text     ; COL is 1-based byte column of the match start
more|limit reached     ; when --max hit
```

Exit: 0 matches, 2 no matches, 1 bad regex (`E|<grep>|0|0|0|re.parse|at=N|what=...`).

Canon invocations:
- find a definition: `llr-grep "\\(fn push" std`
- find uses of a name: `llr-grep "vec\\.push" --glob "*.llr" .`
- literal: `llr-grep "(vec.new <str>)" --fixed std`
