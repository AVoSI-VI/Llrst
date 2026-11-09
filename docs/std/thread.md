# std.thread

A pthread pool, plus one closure started with `spawn`. `start` creates `n` workers. `submit` takes a closure, so a job can capture a binding. When the queue is longer than the worker count, the pool grows by one. `stop` joins the workers. Drop calls `stop`. A second `stop` does nothing. `spawn` returns `(handle T)` and `join` returns the closure's value. Drop of a handle joins and drops that value.

Source: `std/thread.llr`. Import with `(use std.thread)` for module functions; types and methods need no import.

## Types

- `(struct pool (inner (ptr shared)))`
- `(struct handle <T> (id u64) (out (ptr T)) (pack (ptr u8)) (slot (ptr u8)) (done i64))`

## Functions

| name | params | returns |
|------|--------|---------|
| `thread.start` | `((n i64))` | `(res pool str)` |
| `thread.submit` | `((self (&mut pool)) (work (clos () unit)))` | `unit` |
| `thread.workers` | `((self (& pool)))` | `i64` |
| `thread.stop` | `((self (&mut pool)))` | `unit` |
| `thread.spawn <T>` | `((work (clos () T)))` | `(handle T)` |
| `thread.join <T>` | `((h (handle T)))` | `T` |

## Tests

`thread_pool`, `thread_grow`, `thread_names`, `thread_spawn`, `thread_drop`
