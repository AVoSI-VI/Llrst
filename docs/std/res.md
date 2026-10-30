# std.res

Result `(res T E)`: `(res.ok x)` / `(res.err e)`.

Source: `std/res.llr`. Import with `(use std.res)` for module functions; types and methods need no import.

## Types

- `(enum res <T E> (pub (ok T)) (pub (err E)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `res.is_ok` | `((self (& (res T E))))` | `bool` |
| `res.is_err` | `((self (& (res T E))))` | `bool` |
| `res.unwrap` | `((self (res T E)))` | `T` |
| `res.unwrap_err` | `((self (res T E)))` | `E` |

## Tests

`res_basic`
