# std.sexp

S-expression reader and printer for tools (llr-fmt, linters). Keeps comments.

Source: `std/sexp.llr`. Import with `(use std.sexp)` for module functions; types and methods need no import.

## Types

- `(struct tree (nodes (vec node)) (kids (vec i64)) (pub (top (vec i64))))`

## Functions

| name | params | returns |
|------|--------|---------|
| `sexp.read` | `((src (& str)))` | `(res tree str)` |
| `sexp.kind` | `((t (& tree)) (n i64))` | `u8` |
| `sexp.text` | `((t (& tree)) (n i64))` | `(& str)` |
| `sexp.line` | `((t (& tree)) (n i64))` | `i64` |
| `sexp.col` | `((t (& tree)) (n i64))` | `i64` |
| `sexp.count` | `((t (& tree)) (n i64))` | `i64` |
| `sexp.child` | `((t (& tree)) (n i64) (k i64))` | `i64` |
| `sexp.head` | `((t (& tree)) (n i64))` | `(& str)` |
| `sexp.flat` | `((t (& tree)) (n i64))` | `str` |
| `sexp.has_comment` | `((t (& tree)) (n i64))` | `bool` |

## Tests

`sexp_read`
