# std.check_expr

Expression checking. Each expression becomes a texpr: type, callee, builtin id.

Source: `std/check_expr.llr`. Import with `(use std.check_expr)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `check_expr.unit_info` | `()` | `info` |
| `check_expr.none_s` | `()` | `str` |
| `check_expr.node` | `((k u8) (ty str) (callee str) (b i64) (kids (vec texpr)) (e (& expr)))` | `texpr` |
| `check_expr.loc_of` | `((locs (& (vec loc))) (name (& str)))` | `i64` |
| `check_expr.callable` | `((show (& str)))` | `bool` |
| `check_expr.walk` | `((defs (& (vec def))) (srcs (& (vec source))) (from i64) (e (& expr)) (locs (&mut (vec loc))) (out (&mut (vec texpr))) (lo i64) (hi i64))` | `(res info str)` |
| `check_expr.err_at` | `((e (& expr)) (code (& str)) (rest (& str)))` | `str` |
| `check_expr.check_expr` | `((defs (& (vec def))) (srcs (& (vec source))) (from i64) (e (& expr)) (locs (&mut (vec loc))))` | `(res typed str)` |

## Tests

`expr_codes`
