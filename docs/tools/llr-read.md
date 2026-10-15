# llr-read

Numbered lines of a file.

```
llr-read PATH [--from N] [--to N] [--max N]
```

| flag | default | meaning |
|------|---------|---------|
| `--from` | 1 | first line, 1-based |
| `--to` | 0 | last line inclusive; 0 = end of file |
| `--max` | 500 | at most this many lines |

Output grammar:

```
LINE|text          ; one per line
more|NEXT|TOTAL    ; only when truncated: next unread line and total lines
```

Exit: 0; 1 on unreadable file (`E|<read>|0|0|0|io.read|path=...`).

Canon invocation: `llr-read src/main.llr --from 40 --to 80`.
