# std.rand

xorshift64* random numbers. A zero seed is replaced so the state never sticks.

Source: `std/rand.llr`. Import with `(use std.rand)` for module functions; types and methods need no import.

## Types

- `(struct rng (s u64))`

## Functions

| name | params | returns |
|------|--------|---------|
| `rand.seed` | `((s u64))` | `rng` |
| `rand.next` | `((r (&mut rng)))` | `u64` |
| `rand.below` | `((r (&mut rng)) (n i64))` | `i64` |
| `rand.f64` | `((r (&mut rng)))` | `f64` |
| `rand.shuffle <T>` | `((r (&mut rng)) (v (&mut (vec T))))` | `unit` |
| `rand.entropy` | `()` | `rng` |

## Tests

`rand_seq`
