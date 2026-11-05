# std.set

(set K) is a (map K unit). items comes back sorted.

Source: `std/set.llr`. Import with `(use std.set)` for module functions; types and methods need no import.

## Types

- `(struct set <K> (m (map K unit)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `set.new` | `()` | `(set K)` |
| `set.len` | `((self (& (set K))))` | `i64` |
| `set.add <K:Hash:Eq>` | `((self (&mut (set K))) (k K))` | `unit` |
| `set.has <K:Hash:Eq>` | `((self (& (set K))) (k (& K)))` | `bool` |
| `set.del <K:Hash:Eq>` | `((self (&mut (set K))) (k (& K)))` | `bool` |
| `set.items <K:Clone:Ord>` | `((self (& (set K))))` | `(vec K)` |

## Tests

`set_basic`
