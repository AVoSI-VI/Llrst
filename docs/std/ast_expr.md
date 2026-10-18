# std.ast_expr

Expression and type parsing for std.ast. Kinds are documented in std/ast.llr.

Source: `std/ast_expr.llr`. Import with `(use std.ast_expr)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `ast_expr.err_at` | `((t (& tree)) (n i64) (want (& str)) (got (& str)))` | `str` |
| `ast_expr.forms` | `((t (& tree)) (n i64))` | `(vec i64)` |
| `ast_expr.parse_ty` | `((t (& tree)) (n i64))` | `(res ty str)` |
| `ast_expr.parse_exprs` | `((t (& tree)) (ids (& (vec i64))) (from i64) (out (&mut (vec expr))))` | `(res unit str)` |
| `ast_expr.parse_expr` | `((t (& tree)) (n i64))` | `(res expr str)` |
