# std.check_more

Impl shape and generic bounds for llr-check. The compiler in self/ is the reference.

Source: `std/check_more.llr`. Import with `(use std.check_more)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `check_more.impl_shape` | `((srcs (& (vec source))) (main i64))` | `(opt str)` |
| `check_more.bounds` | `((srcs (& (vec source))) (d (& def)) (e (& expr)) (kids (& (vec texpr))))` | `(opt str)` |
| `check_more.clos_show` | `((e (& expr)))` | `str` |
| `check_more.clos_args` | `((srcs (& (vec source))) (d (& def)) (kids (& (vec texpr))))` | `str` |
| `check_more.ctor_hide` | `((srcs (& (vec source))) (from i64) (d (& def)) (it (& item)) (e (& expr)))` | `str` |

## Tests

`more_impl_shape`, `more_bound`
