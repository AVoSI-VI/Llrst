# std.cmp

`Eq`, `Ord`, `Hash`, `Clone`, `Show`, and `From`. `Ord.cmp` is negative when the first value comes first. `i64` and `str` implement the first five. `Show.show` renders a value for `assert_eq`. `From.from` converts the argument into `Self`; `(try e)` calls it when a `res` error differs from the function's error.

`(Eq.eq a b)` is a direct call. `(& (dyn Eq))` is a fat pointer. `Clone` and `From` return `Self`, so they are not trait objects.

Source: `std/cmp.llr`. Traits are prelude. Import with `(use std.cmp)` is not required for `Eq`, `Ord`, `Hash`, `Clone`, or `From`.

## Traits

| name | params | returns |
|------|--------|---------|
| `Eq.eq` | `((& Self) (& Self))` | `bool` |
| `Ord.cmp` | `((& Self) (& Self))` | `i64` |
| `Hash.hash` | `((& Self))` | `i64` |
| `Clone.clone` | `((& Self))` | `Self` |
| `From.from` | `(Self)` | `Self` |
| `Show.show` | `((& Self))` | `str` |

## Functions

| name | params | returns |
|------|--------|---------|

## Tests

`cmp_traits`, `cmp_ints`, `cmp_floats`, `cmp_coll`
