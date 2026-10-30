# std.check_ty

Type display and type-name checks for std.check.

Source: `std/check_ty.llr`. Import with `(use std.check_ty)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `check_ty.show_ty` | `((t (& ty)) (gens (& (vec str))) (args (& (vec info))))` | `str` |
| `check_ty.gen_err` | `((line i64) (col i64) (len i64) (name (& str)) (want i64) (got i64) (named bool))` | `str` |
| `check_ty.check_ty` | `((defs (& (vec def))) (srcs (& (vec source))) (from i64) (t (& ty)) (gens (& (vec str))))` | `(opt str)` |
| `check_ty.info_ty` | `((defs (& (vec def))) (srcs (& (vec source))) (scope (& str)) (from i64) (t (& ty)) (gens (& (vec str))) (args (& (vec info))))` | `info` |

## Tests

`show_clos`
