# std.map

Hash map `(map K V)`. Each call passes a hash `(fn ((& K)) i64)` and an equality `(fn ((& K) (& K)) bool)`. Helpers cover `str` and `i64`.

Source: `std/map.llr`. Import with `(use std.map)` for module functions; types and methods need no import.

## Types

- `(struct entry <K V> (k K) (v V) (next i64) (alive bool))`
- `(struct map <K V> (heads (vec i64)) (ents (vec (entry K V))) (len i64))`

## Functions

| name | params | returns |
|------|--------|---------|
| `map.hash_str` | `((s (& str)))` | `i64` |
| `map.eq_str` | `((a (& str)) (b (& str)))` | `bool` |
| `map.hash_int` | `((k (& i64)))` | `i64` |
| `map.eq_int` | `((a (& i64)) (b (& i64)))` | `bool` |
| `map.clone_int` | `((k (& i64)))` | `i64` |
| `map.new` | `()` | `(map K V)` |
| `map.len` | `((self (& (map K V))))` | `i64` |
| `map.bucket` | `((self (& (map K V))) (k (& K)) (hash (fn ((& K)) i64)))` | `i64` |
| `map.find` | `((self (& (map K V))) (k (& K)) (hash (fn ((& K)) i64)) (eq (fn ((& K) (& K)) bool)))` | `i64` |
| `map.has` | `((self (& (map K V))) (k (& K)) (hash (fn ((& K)) i64)) (eq (fn ((& K) (& K)) bool)))` | `bool` |
| `map.get` | `((self (& (map K V))) (k (& K)) (hash (fn ((& K)) i64)) (eq (fn ((& K) (& K)) bool)))` | `(opt (& V))` |
| `map.get_mut` | `((self (&mut (map K V))) (k (& K)) (hash (fn ((& K)) i64)) (eq (fn ((& K) (& K)) bool)))` | `(opt (&mut V))` |
| `map.relink` | `((self (&mut (map K V))) (hash (fn ((& K)) i64)))` | `unit` |
| `map.rehash` | `((self (&mut (map K V))) (hash (fn ((& K)) i64)))` | `unit` |
| `map.compact` | `((self (&mut (map K V))) (hash (fn ((& K)) i64)))` | `unit` |
| `map.put` | `((self (&mut (map K V))) (k K) (v V) (hash (fn ((& K)) i64)) (eq (fn ((& K) (& K)) bool)))` | `unit` |
| `map.del` | `((self (&mut (map K V))) (k (& K)) (hash (fn ((& K)) i64)) (eq (fn ((& K) (& K)) bool)))` | `bool` |
| `map.keys` | `((self (& (map K V))) (clone (fn ((& K)) K)) (cmp (fn ((& K) (& K)) i64)))` | `(vec K)` |

## Tests

`map_basic`, `map_int`, `map_compact`
