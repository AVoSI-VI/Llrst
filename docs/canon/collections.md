# Canon: collections

## Operations over T

There are no traits. An operation that depends on `T` or `K` takes a named `(fn ...)` on that call. The pointer is not stored and cannot capture a value. Invoke it with `(call f a b)`.

| operation | pointer | helpers |
|-----------|---------|---------|
| `vec.has` | `(fn ((& T) (& T)) bool)` | `vec.eq_str`, `vec.eq_int` |
| `vec.index` | `(fn ((& T) (& T)) bool)` | `vec.eq_str`, `vec.eq_int` |
| `vec.sort` | `(fn ((& T) (& T)) i64)`. Negative or zero means the first argument comes first; zero keeps the earlier element | `vec.cmp_str`, `vec.cmp_int` |
| map hash | `(fn ((& K)) i64)` | `map.hash_str`, `map.hash_int` |
| map eq | `(fn ((& K) (& K)) bool)` | `map.eq_str`, `map.eq_int` |
| `map.keys` | `(fn ((& K)) K)` plus the sort pointer | `str.clone` or `map.clone_int`, and `vec.cmp_str` or `vec.cmp_int` |

## vec

| task | form |
|------|------|
| create | `(vec.new <T>)` or let the expected type infer it |
| push | `(vec.push (&mut v) x)` moves `x` |
| length | `(vec.len (& v))` |
| read | `(vec.get (& v) i)` returns `(& T)`, panics oob |
| read mutable | `(vec.get_mut (&mut v) i)` returns `(&mut T)` |
| overwrite | `(vec.set (&mut v) i x)` drops old |
| pop | `(vec.pop (&mut v))` returns `(opt T)` |
| remove | `(vec.remove (&mut v) i)` returns `T`, shifts |
| iterate | `(for i 0 (vec.len (& v)) (let x (vec.get (& v) i)) ...)` |
| contains | `(vec.has (& v) x vec.eq_str)` or `vec.eq_int` |
| index of | `(vec.index (& v) x vec.eq_str)` returns `(opt i64)` |
| sort | `(vec.sort (&mut v) vec.cmp_str)` or `vec.cmp_int`, stable. A custom order is a named `(fn ((& T) (& T)) i64)` |
| clone | `(vec.clone_str (& v))` |
| reverse | `(vec.reverse (&mut v))` |
| insert | `(vec.insert (&mut v) i x)` |
| last | `(vec.last (& v))` returns `(opt (& T))` |
| clear | `(vec.clear (&mut v))` |
| take all | `(vec.take (&mut v))` returns the vec, leaves empty |

No map or filter in v1. Write the loop. `vec.any` is not a method. A `(fn ((& T)) bool)` cannot capture a needle, so the loop stays at the call site. A `(clos ...)` can capture a binding, and it is not a `(fn ...)`.

## map

Keys are any `K`. Each call passes `(fn ((& K)) i64)` and `(fn ((& K) (& K)) bool)`. They are not stored. Str uses `map.hash_str` and `map.eq_str`. `i64` uses `map.hash_int` and `map.eq_int`.

| task | form |
|------|------|
| create | `(map.new <str V>)` or `(map.new <i64 V>)` |
| insert | `(map.put (&mut m) k v map.hash_str map.eq_str)` moves `k`, replaces an existing value |
| get | `(map.get (& m) k map.hash_str map.eq_str)` returns `(opt (& V))` |
| get mutable | `(map.get_mut (&mut m) k map.hash_str map.eq_str)` returns `(opt (&mut V))` |
| has | `(map.has (& m) k map.hash_str map.eq_str)` |
| remove | `(map.del (&mut m) k map.hash_str map.eq_str)` returns `bool` |
| size | `(map.len (& m))` |
| keys | `(map.keys (& m) str.clone vec.cmp_str)` returns a sorted `(vec K)`. `i64` keys use `map.clone_int` and `vec.cmp_int` |
