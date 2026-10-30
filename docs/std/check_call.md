# std.check_call

Calls, trait methods, and field access for llr-check.

Source: `std/check_call.llr`. Import with `(use std.check_call)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `check_call.check_call` | `((defs (& (vec def))) (srcs (& (vec source))) (from i64) (e (& expr)) (locs (&mut (vec loc))))` | `(res typed str)` |
| `check_call.check_field` | `((defs (& (vec def))) (srcs (& (vec source))) (from i64) (e (& expr)) (locs (&mut (vec loc))))` | `(res typed str)` |
