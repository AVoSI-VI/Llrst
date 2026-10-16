# std.vec

Growable array `(vec T)`.

Source: `std/vec.llr`. Import with `(use std.vec)` for module functions; types and methods need no import.

## Types

- `(struct vec <T> (p (ptr T)) (len i64) (cap i64))`

## Functions

| name | params | returns |
|------|--------|---------|
| `vec.new` | `()` | `(vec T)` |
| `vec.drop` | `((self (&mut (vec T))))` | `unit` |
| `vec.len` | `((self (& (vec T))))` | `i64` |
| `vec.is_empty` | `((self (& (vec T))))` | `bool` |
| `vec.reserve` | `((self (&mut (vec T))) (extra i64))` | `unit` |
| `vec.push` | `((self (&mut (vec T))) (x T))` | `unit` |
| `vec.check` | `((self (& (vec T))) (i i64))` | `unit` |
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
| `vec.has` | `((self (& (vec T))) (x (& T)) (eq (fn ((& T) (& T)) bool)))` | `bool` |
| `vec.sort` | `((self (&mut (vec T))) (cmp (fn ((& T) (& T)) i64)))` | `unit` |
| `vec.cmp_str` | `((a (& str)) (b (& str)))` | `i64` |
| `vec.cmp_int` | `((a (& i64)) (b (& i64)))` | `i64` |
| `vec.eq_str` | `((a (& str)) (b (& str)))` | `bool` |
| `vec.eq_int` | `((a (& i64)) (b (& i64)))` | `bool` |
| `vec.index_str` | `((v (& (vec str))) (x (& str)))` | `(opt i64)` |
| `vec.clone_str` | `((v (& (vec str))))` | `(vec str)` |

## Tests

`vec_push_get`, `vec_str_owned`, `vec_sort_stable`
