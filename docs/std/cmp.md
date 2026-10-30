# std.cmp

`Eq`, `Ord`, `Hash`, and `Clone`. `Ord.cmp` is negative when the first value comes first. `i64` and `str` implement all four.

`(Eq.eq a b)` is a direct call. `(& (dyn Eq))` is a fat pointer. `Clone` returns `Self`, so it is not a trait object.

Source: `std/cmp.llr`. Traits are prelude. Import with `(use std.cmp)` is not required for `Eq`, `Ord`, `Hash`, or `Clone`.

## Traits

| name | params | returns |
|------|--------|---------|
| `Eq.eq` | `((& Self) (& Self))` | `bool` |
| `Ord.cmp` | `((& Self) (& Self))` | `i64` |
| `Hash.hash` | `((& Self))` | `i64` |
| `Clone.clone` | `((& Self))` | `Self` |

## Functions

| name | params | returns |
|------|--------|---------|

## Tests

`cmp_traits`
