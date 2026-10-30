# std.opt

Optional `(opt T)`: `(opt.some x)` / `(opt.none)`.

Source: `std/opt.llr`. Import with `(use std.opt)` for module functions; types and methods need no import.

## Types

- `(enum opt <T> (pub (some T)) (pub (none)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `opt.is_some` | `((self (& (opt T))))` | `bool` |
| `opt.is_none` | `((self (& (opt T))))` | `bool` |
| `opt.unwrap` | `((self (opt T)))` | `T` |
| `opt.or` | `((self (opt T)) (d T))` | `T` |

## Tests

`opt_basic`
