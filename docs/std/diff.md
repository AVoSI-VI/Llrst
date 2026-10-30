# std.diff

Line diff. Shared ends are exact. The middle is a minimal LCS.

Source: `std/diff.llr`. Import with `(use std.diff)` for module functions; types and methods need no import.

## Types

- `(struct op (pub (kind u8)) (pub (a i64)) (pub (b i64)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `diff.ops` | `((a (& (vec str))) (b (& (vec str))))` | `(vec op)` |
| `diff.lines` | `((a (& (vec str))) (b (& (vec str))) (context i64))` | `(vec str)` |
| `diff.stat` | `((a (& (vec str))) (b (& (vec str))))` | `(vec i64)` |

## Tests

`diff_helpers`, `diff_lines`, `diff_middle`, `diff_bound`
