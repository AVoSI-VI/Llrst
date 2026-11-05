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
| `opt.map <U>` | `((self (opt T)) (f (clos ((& T)) U)))` | `(opt U)` |
| `opt.and_then <U>` | `((self (opt T)) (f (clos ((& T)) (opt U))))` | `(opt U)` |
| `opt.ok_or <E>` | `((self (opt T)) (e E))` | `(res T E)` |
| `opt.expect` | `((self (opt T)) (msg (& str)))` | `T` |
| `opt.unwrap_or_else` | `((self (opt T)) (f (clos () T)))` | `T` |
| `opt.take` | `((self (&mut (opt T))))` | `(opt T)` |

## Tests

`opt_basic`
