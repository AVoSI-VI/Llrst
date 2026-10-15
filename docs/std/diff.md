# std.diff

Line diff by LCS, pipe records.

Source: `std/diff.llr`. Import with `(use std.diff)` for module functions; types and methods need no import.

## Types

- `(struct op (kind u8) (a i64) (b i64))`

## Functions

| name | params | returns |
|------|--------|---------|
| `diff.ops` | `((a (& (vec str))) (b (& (vec str))))` | `(vec op)` |
| `diff.lines` | `((a (& (vec str))) (b (& (vec str))) (context i64))` | `(vec str)` |
| `diff.stat` | `((a (& (vec str))) (b (& (vec str))))` | `(vec i64)` |

## Tests

`diff_lines`
