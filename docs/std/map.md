# std.map

Hash map `(map V)` with `str` keys.

Source: `std/map.llr`. Import with `(use std.map)` for module functions; types and methods need no import.

## Types

- `(struct entry <V> (k str) (v V) (next i64) (alive bool))`
- `(struct map <V> (heads (vec i64)) (ents (vec (entry V))) (len i64))`

## Functions

| name | params | returns |
|------|--------|---------|
| `map.new` | `()` | `(map V)` |
| `map.len` | `((self (& (map V))))` | `i64` |
| `map.bucket` | `((self (& (map V))) (k (& str)))` | `i64` |
| `map.find` | `((self (& (map V))) (k (& str)))` | `i64` |
| `map.has` | `((self (& (map V))) (k (& str)))` | `bool` |
| `map.get` | `((self (& (map V))) (k (& str)))` | `(opt (& V))` |
| `map.get_mut` | `((self (&mut (map V))) (k (& str)))` | `(opt (&mut V))` |
| `map.rehash` | `((self (&mut (map V))))` | `unit` |
| `map.put` | `((self (&mut (map V))) (k str) (v V))` | `unit` |
| `map.del` | `((self (&mut (map V))) (k (& str)))` | `bool` |
| `map.keys` | `((self (& (map V))))` | `(vec str)` |

## Tests

`map_basic`
