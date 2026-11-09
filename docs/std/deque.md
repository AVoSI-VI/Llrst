# std.deque

Ring of opt slots so drop frees only the live elements.

Source: `std/deque.llr`. Import with `(use std.deque)` for module functions; types and methods need no import.

## Types

- `(struct deque <T> (buf (vec (opt T))) (head i64) (len i64))`

## Functions

| name | params | returns |
|------|--------|---------|
| `deque.new` | `()` | `(deque T)` |
| `deque.len` | `((self (& (deque T))))` | `i64` |
| `deque.push_back` | `((self (&mut (deque T))) (x T))` | `unit` |
| `deque.push_front` | `((self (&mut (deque T))) (x T))` | `unit` |
| `deque.pop_front` | `((self (&mut (deque T))))` | `(opt T)` |
| `deque.pop_back` | `((self (&mut (deque T))))` | `(opt T)` |
| `deque.get` | `((self (& (deque T))) (i i64))` | `(& T)` |

## Tests

`deque_ring`
