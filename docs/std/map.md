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
| `map.values` | `((self (& (map K V))))` | `(vec (& V))` |
| `map.each` | `((self (& (map K V))) (f (clos ((& K) (& V)) unit)))` | `unit` |
| `map.take` | `((self (&mut (map K V))))` | `(map K V)` |
| `map.clear` | `((self (&mut (map K V))))` | `unit` |
| `map.get_or_insert <K:Hash:Eq>` | `((self (&mut (map K V))) (k K) (make (clos () V)))` | `(&mut V)` |

## Tests

`map_slots`, `map_basic`, `map_int`, `map_compact`, `map_shrink`, `map_values`, `map_each`, `map_edit`
