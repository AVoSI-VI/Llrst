# Canon: collections

## Operations over T

`Eq`, `Ord`, `Hash`, `Clone`, and `Show` are traits in `std/cmp.llr`. A call is `(Eq.eq a b)`. `Ord.cmp` is negative when the first value comes first. The integer types, `str`, and `bool` implement `Eq`, `Ord`, `Hash`, `Clone`, and `Show`; the `i128` and `u128` impls are in `std/cmp_wide.llr`. `f64` and `f32` implement `Eq`, `Clone`, and `Show`; `Eq` is `==`, so NaN is not equal to NaN. `(vec T)`, `(opt T)`, and `(res T E)` implement those traits when their arguments do (`Ord` on `res` needs both `T: Ord` and `E: Ord`). `vec.has` and `vec.index` need `T: Eq`. `vec.sort` needs `T: Ord`. `vec.clone` needs `T: Clone`. Map keys need `K: Hash` and `K: Eq`. `map.keys` and `set.items` need `K: Clone` and `K: Ord`.

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
| sort | `(vec.sort (&mut v))` when `T: Ord`, stable |
| sort by | `(vec.sort_by (&mut v) cmp)` stable. `cmp` is `(clos ((& T) (& T)) i64)`; `0` keeps the left run |
| clone | `(vec.clone (& v))` when `T: Clone`, or `(Clone.clone (& v))` |
| extend | `(vec.extend (&mut v) (& src))` clones each element when `T: Clone` |
| filled | `(vec.filled n (& x))` when `T: Clone`. `n < 0` panics |
| resize | `(vec.resize (&mut v) n (& x))` truncates or clones `x` up to `n` |
| truncate | `(vec.truncate (&mut v) n)` drops the tail. `n < 0` panics |
| first | `(vec.first (& v))` returns `(opt (& T))` |
| swap remove | `(vec.swap_remove (&mut v) i)` returns `T` |
| equal | `(vec.eq (& a) (& b))` when `T: Eq` |
| dedup | `(vec.dedup (&mut v))` drops consecutive equals when `T: Eq` |
| min / max | `(vec.min (& v))` `(vec.max (& v))` return `(opt (& T))` when `T: Ord` |
| retain | `(vec.retain (&mut v) pred)` keeps elements where `pred` is `(clos ((& T)) bool)` |
| each | `(vec.each (& v) f)` where `f` is `(clos ((& T)) unit)` |
| map | `(vec.map (& v) f)` builds a new vec. `f` is `(clos ((& T)) U)` |
| filter | `(vec.filter (& v) pred)` clones kept elements when `T: Clone` |
| reverse | `(vec.reverse (&mut v))` |
| insert | `(vec.insert (&mut v) i x)` |
| last | `(vec.last (& v))` returns `(opt (& T))` |
| clear | `(vec.clear (&mut v))` |
| take all | `(vec.take (&mut v))` returns the vec, leaves empty |
| bytes for an extern | `(vec.ptr (& v))` returns `(ptr T)`. An empty vec is null |

A predicate that captures a binding is `(clos ...)`, passed to `vec.any`, `vec.find`, `vec.each`, `vec.map`, `vec.filter`, or `vec.retain`. It is not a `(fn ...)`.

## arr

`(arr T N)` is `N` elements of `T` inline. No `Eq`, `Clone`, or `Show`: compare and copy by element.

| task | form |
|------|------|
| create | `(arr.of x y z)`, or `(let a (arr T N) (arr.of ...))` when the element type should be stated |
| fill | `(arr.filled x)` when the expected type is `(arr T N)` and `T` is `copy`, or `(arr.filled <(arr T N)> x)` |
| length | `(arr.len (& a))` |
| read | `(arr.get (& a) i)` returns `(& T)`, panics oob |
| read mutable | `(arr.get_mut (&mut a) i)` returns `(&mut T)` |
| overwrite | `(arr.set (&mut a) i x)` drops old |
| pointer for an extern | `(as (ptr T) (& a))` |
| iterate | `(for i 0 (arr.len (& a)) (let x (arr.get (& a) i)) ...)` |

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
| values | `(map.values (& m))` returns `(vec (& V))` of live entries, storage order |
| each | `(map.each (& m) f)` over live entries, storage order. `f` is `(clos ((& K) (& V)) unit)` |
| clear | `(map.clear (&mut m))` drops every entry |
| take | `(map.take (&mut m))` returns the map and leaves it empty |
| get or insert | `(map.get_or_insert (&mut m) k (clos () V ...))` returns `(&mut V)`. The closure runs only when `k` is absent |

## set

`(set K)` is a `(map K unit)`. `add`, `has`, and `del` need `K: Hash: Eq`.

| task | form |
|------|------|
| create | `(set.new <K>)` |
| add | `(set.add (&mut s) k)` |
| has | `(set.has (& s) k)` |
| remove | `(set.del (&mut s) k)` returns `bool` |
| size | `(set.len (& s))` |
| items | `(set.items (& s))` returns a sorted `(vec K)` when `K: Clone: Ord` |

## deque

`(deque T)` is a ring of `(opt T)`. `get` panics out of range. `pop_front` and `pop_back` return `(opt T)`.

| task | form |
|------|------|
| create | `(deque.new <T>)` |
| push back | `(deque.push_back (&mut d) x)` |
| push front | `(deque.push_front (&mut d) x)` |
| pop front | `(deque.pop_front (&mut d))` |
| pop back | `(deque.pop_back (&mut d))` |
| length | `(deque.len (& d))` |
| read | `(deque.get (& d) i)` returns `(& T)` |
