# Canon: collections

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
| index of | `(vec.index_str (& v) x)` returns `(opt i64)` |
| sort | `(vec.sort (&mut v) vec.cmp_str)` or `vec.cmp_int`, stable. A custom order is a named `(fn ((& T) (& T)) i64)` |
| clone | `(vec.clone_str (& v))` |
| reverse | `(vec.reverse (&mut v))` |
| insert | `(vec.insert (&mut v) i x)` |
| last | `(vec.last (& v))` returns `(opt (& T))` |
| clear | `(vec.clear (&mut v))` |
| take all | `(vec.take (&mut v))` returns the vec, leaves empty |

No map/filter/closures in v1. Write the loop.

## map

Keys are always `str` (use `(str.from_int n)` for integer keys). Values: any.

| task | form |
|------|------|
| create | `(map.new <V>)` |
| insert | `(map.put (&mut m) k v)` moves `k`, replaces an existing value |
| get | `(map.get (& m) k)` returns `(opt (& V))` |
| get mutable | `(map.get_mut (&mut m) k)` returns `(opt (&mut V))` |
| has | `(map.has (& m) k)` |
| remove | `(map.del (&mut m) k)` returns `bool` |
| size | `(map.len (& m))` |
| keys | `(map.keys (& m))` returns `(vec str)` sorted, cloned |
