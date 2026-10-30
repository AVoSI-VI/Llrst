# std.vec

Growable array `(vec T)`.

Source: `std/vec.llr`. Import with `(use std.vec)` for module functions; types and methods need no import.

## Types

- `(struct vec <T> (p (ptr T)) (len i64) (cap i64))`

## Functions

| name | params | returns |
|------|--------|---------|
| `vec.new` | `()` | `(vec T)` |
| `vec.len` | `((self (& (vec T))))` | `i64` |
| `vec.is_empty` | `((self (& (vec T))))` | `bool` |
| `vec.reserve` | `((self (&mut (vec T))) (extra i64))` | `unit` |
| `vec.push` | `((self (&mut (vec T))) (x T))` | `unit` |
| `vec.get` | `((self (& (vec T))) (i i64))` | `(& T)` |
| `vec.get_mut` | `((self (&mut (vec T))) (i i64))` | `(&mut T)` |
| `vec.last` | `((self (& (vec T))))` | `(opt (& T))` |
| `vec.set` | `((self (&mut (vec T))) (i i64) (x T))` | `unit` |
| `vec.pop` | `((self (&mut (vec T))))` | `(opt T)` |
| `vec.remove` | `((self (&mut (vec T))) (i i64))` | `T` |
| `vec.insert` | `((self (&mut (vec T))) (i i64) (x T))` | `unit` |
| `vec.swap` | `((self (&mut (vec T))) (i i64) (j i64))` | `unit` |
| `vec.clear` | `((self (&mut (vec T))))` | `unit` |
| `vec.take` | `((self (&mut (vec T))))` | `(vec T)` |
| `vec.append` | `((self (&mut (vec T))) (other (vec T)))` | `unit` |
| `vec.reverse` | `((self (&mut (vec T))))` | `unit` |
| `vec.has <T:Eq>` | `((self (& (vec T))) (x (& T)))` | `bool` |
| `vec.index <T:Eq>` | `((self (& (vec T))) (x (& T)))` | `(opt i64)` |
| `vec.any <T>` | `((self (& (vec T))) (pred (clos ((& T)) bool)))` | `bool` |
| `vec.find <T>` | `((self (& (vec T))) (pred (clos ((& T)) bool)))` | `(opt i64)` |
| `vec.sort <T:Ord>` | `((self (&mut (vec T))))` | `unit` |
| `vec.clone_str` | `((v (& (vec str))))` | `(vec str)` |

## Tests

`vec_push_get`, `vec_str_owned`, `vec_sort_stable`, `vec_edit`, `vec_index`, `vec_find`
