# std.check

Type and name checker over std.ast. Borrow checking stays in llrc.

Source: `std/check.llr`. Import with `(use std.check)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `check.check` | `((path (& str)))` | `(opt str)` |

## Tests

`check_mismatch`
