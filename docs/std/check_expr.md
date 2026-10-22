# std.check_expr

Expression checking. Each expression becomes a texpr: type, callee, builtin id.

Source: `std/check_expr.llr`. Import with `(use std.check_expr)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `check_expr.check_expr` | `((defs (& (vec def))) (srcs (& (vec source))) (from i64) (e (& expr)) (locs (&mut (vec loc))))` | `(res typed str)` |
