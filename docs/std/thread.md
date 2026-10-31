# std.thread

A pthread pool. `start` creates `n` workers. `submit` takes a closure, so a job can capture a binding. When the queue is longer than the worker count, the pool grows by one. `stop` joins the workers. Drop calls `stop`. A second `stop` does nothing.

Source: `std/thread.llr`. Import with `(use std.thread)` for module functions; types and methods need no import.

## Types

- `(struct pool (inner (ptr shared)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `thread.start` | `((n i64))` | `(res pool str)` |
| `thread.submit` | `((self (&mut pool)) (work (clos () unit)))` | `unit` |
| `thread.workers` | `((self (& pool)))` | `i64` |
| `thread.stop` | `((self (&mut pool)))` | `unit` |

## Tests

`thread_pool`, `thread_grow`, `thread_names`
