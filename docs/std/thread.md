# std.thread

A fixed pthread pool. The thread count is chosen at `start` and does not grow. A job is a function pointer plus a context pointer. The pointer cannot capture a binding. `stop` joins the threads. Drop calls `stop`. A second `stop` does nothing.

Source: `std/thread.llr`. Import with `(use std.thread)` for module functions; types and methods need no import.

## Types

- `(struct pool (inner (ptr shared)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `thread.start` | `((n i64))` | `(res pool str)` |
| `thread.submit` | `((self (&mut pool)) (work (fn ((ptr u8)) unit)) (ctx (ptr u8)))` | `unit` |
| `thread.stop` | `((self (&mut pool)))` | `unit` |

## Tests

`thread_pool`, `thread_names`
