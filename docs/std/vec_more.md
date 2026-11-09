# std.vec_more

More vec methods. vec.llr keeps sort; this file is a second impl.

Source: `std/vec_more.llr`. Import with `(use std.vec_more)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `vec.clone <T:Clone>` | `((self (& (vec T))))` | `(vec T)` |
| `vec.extend <T:Clone>` | `((self (&mut (vec T))) (src (& (vec T))))` | `unit` |
| `vec.filled <T:Clone>` | `((n i64) (x (& T)))` | `(vec T)` |
| `vec.resize <T:Clone>` | `((self (&mut (vec T))) (n i64) (x (& T)))` | `unit` |
| `vec.truncate` | `((self (&mut (vec T))) (n i64))` | `unit` |
| `vec.first` | `((self (& (vec T))))` | `(opt (& T))` |
| `vec.swap_remove` | `((self (&mut (vec T))) (i i64))` | `T` |
| `vec.eq <T:Eq>` | `((self (& (vec T))) (o (& (vec T))))` | `bool` |
| `vec.dedup <T:Eq>` | `((self (&mut (vec T))))` | `unit` |
| `vec.min <T:Ord>` | `((self (& (vec T))))` | `(opt (& T))` |
| `vec.max <T:Ord>` | `((self (& (vec T))))` | `(opt (& T))` |
| `vec.sort_by <T>` | `((self (&mut (vec T))) (cmp (clos ((& T) (& T)) i64)))` | `unit` |
| `vec.retain <T>` | `((self (&mut (vec T))) (pred (clos ((& T)) bool)))` | `unit` |
| `vec.each <T>` | `((self (& (vec T))) (f (clos ((& T)) unit)))` | `unit` |
| `vec.map <U>` | `((self (& (vec T))) (f (clos ((& T)) U)))` | `(vec U)` |
| `vec.filter <T:Clone>` | `((self (& (vec T))) (pred (clos ((& T)) bool)))` | `(vec T)` |

## Tests

`vec_clone`, `vec_resize`, `vec_eq_ends`, `vec_sort_by`, `vec_clos`
