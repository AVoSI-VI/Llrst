# std.check_load

Where llr-check finds std, and use-path errors that have no source body.

Source: `std/check_load.llr`. Import with `(use std.check_load)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `check_load.slash` | `((p (& str)))` | `str` |
| `check_load.root` | `((over (& str)) (file (& str)))` | `(res str str)` |
| `check_load.bad_use` | `((srcs (& (vec source))) (from i64))` | `(opt str)` |
