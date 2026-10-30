# std.map

Hash map `(map K V)`. Keys need `K: Hash` and `K: Eq`. `map.keys` needs `K: Clone` and `K: Ord`.

Source: `std/map.llr`. Import with `(use std.map)` for module functions; types and methods need no import.

## Types

- `(struct map <K V> (heads (vec i64)) (ents (vec (entry K V))) (len i64))`

## Functions

| name | params | returns |
|------|--------|---------|
| `map.new` | `()` | `(map K V)` |
| `map.len` | `((self (& (map K V))))` | `i64` |
| `map.has <K:Hash:Eq>` | `((self (& (map K V))) (k (& K)))` | `bool` |
| `map.get <K:Hash:Eq>` | `((self (& (map K V))) (k (& K)))` | `(opt (& V))` |
| `map.get_mut <K:Hash:Eq>` | `((self (&mut (map K V))) (k (& K)))` | `(opt (&mut V))` |
| `map.put <K:Hash:Eq>` | `((self (&mut (map K V))) (k K) (v V))` | `unit` |
| `map.del <K:Hash:Eq>` | `((self (&mut (map K V))) (k (& K)))` | `bool` |
| `map.keys <K:Clone:Ord>` | `((self (& (map K V))))` | `(vec K)` |

## Tests

`map_slots`, `map_basic`, `map_int`, `map_compact`, `map_shrink`
