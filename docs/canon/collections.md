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
| contains | `(vec.has (& v) (& x))` for `str` and primitives |
| sort | `(vec.sort (&mut v))` for `str` and primitives, stable |
| clear | `(vec.clear (&mut v))` |
| take all | `(vec.take (&mut v))` returns the vec, leaves empty |

No map/filter/closures in v1. Write the loop.

## map

Keys: `str`, `i64`, `u64`. Values: any.

| task | form |
|------|------|
| create | `(map.new <K V>)` |
| insert | `(map.put (&mut m) k v)` replaces |
| get | `(map.get (& m) (& k))` returns `(opt (& V))` |
| has | `(map.has (& m) (& k))` |
| remove | `(map.del (&mut m) (& k))` returns `(opt V)` |
| size | `(map.len (& m))` |
| keys | `(map.keys (& m))` returns `(vec K)` sorted, keys cloned |
