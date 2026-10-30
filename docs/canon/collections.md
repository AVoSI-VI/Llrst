# Canon: collections

## Operations over T

`Eq`, `Ord`, `Hash`, and `Clone` are traits in `std/cmp.llr`. A call is `(Eq.eq a b)`. `Ord.cmp` is negative when the first value comes first. `vec.has` and `vec.index` need `T: Eq`. `vec.sort` needs `T: Ord`. Map keys need `K: Hash` and `K: Eq`. `map.keys` needs `K: Clone` and `K: Ord`.

`(& (dyn Eq))` is a fat pointer. Pass `(& n)` where `(& (dyn Eq))` is expected and `n`'s type implements `Eq`. `Clone` returns `Self`, so it is not a trait object.

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
| contains | `(vec.has (& v) x)` when `T: Eq` |
| index of | `(vec.index (& v) x)` returns `(opt i64)` when `T: Eq` |
| any match | `(vec.any (& v) pred)` where `pred` is `(clos ((& T)) bool)` and may capture a needle |
| first match | `(vec.find (& v) pred)` returns `(opt i64)`. Same closure as `any` |
| sort | `(vec.sort (&mut v))` when `T: Ord`, stable. A custom order is `(impl Ord item ...)` |
| clone | `(vec.clone_str (& v))` |
| reverse | `(vec.reverse (&mut v))` |
| insert | `(vec.insert (&mut v) i x)` |
| last | `(vec.last (& v))` returns `(opt (& T))` |
| clear | `(vec.clear (&mut v))` |
| take all | `(vec.take (&mut v))` returns the vec, leaves empty |

No map or filter in v1. Write the loop. A predicate that captures a binding is `(clos ...)`, passed to `vec.any` or `vec.find`. It is not a `(fn ...)`.

## map

Keys are any `K: Hash: Eq`. Str and `i64` implement those traits.

| task | form |
|------|------|
| create | `(map.new <str V>)` or `(map.new <i64 V>)` |
| insert | `(map.put (&mut m) k v)` moves `k`, replaces an existing value |
| get | `(map.get (& m) k)` returns `(opt (& V))` |
| get mutable | `(map.get_mut (&mut m) k)` returns `(opt (&mut V))` |
| has | `(map.has (& m) k)` |
| remove | `(map.del (&mut m) k)` returns `bool` |
| size | `(map.len (& m))` |
| keys | `(map.keys (& m))` returns a sorted `(vec K)` when `K: Clone: Ord` |
