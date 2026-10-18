# std.diff

Line diff. Shared ends are exact. The middle is an LCS when it is at most 4096 cells, otherwise one replace hunk.

Source: `std/diff.llr`. Import with `(use std.diff)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `diff.ops` | `((a (& (vec str))) (b (& (vec str))))` | `(vec op)` |
| `diff.lines` | `((a (& (vec str))) (b (& (vec str))) (context i64))` | `(vec str)` |
| `diff.stat` | `((a (& (vec str))) (b (& (vec str))))` | `(vec i64)` |

## Tests

`diff_lines`, `diff_middle`, `diff_bound`
