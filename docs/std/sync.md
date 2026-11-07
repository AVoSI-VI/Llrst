# std.sync

A mutex whose guard cannot escape, and a channel over a condition variable.

Source: `std/sync.llr`. Import with `(use std.sync)` for module functions; types and methods need no import.

## Types

- `(struct mutex <T> (gate (ptr u8)) (val T))`
- `(struct chan <T> (inner (ptr (state T))))`

## Functions

| name | params | returns |
|------|--------|---------|
| `mutex.new` | `((v T))` | `(mutex T)` |
| `mutex.lock` | `((self (&mut (mutex T))) (body (clos ((&mut T)) unit)))` | `unit` |
| `chan.new` | `()` | `(chan T)` |
| `chan.clone` | `((self (& (chan T))))` | `(chan T)` |
| `chan.send` | `((self (& (chan T))) (v T))` | `bool` |
| `chan.recv` | `((self (& (chan T))))` | `(opt T)` |
| `chan.try_recv` | `((self (& (chan T))))` | `(opt T)` |
| `chan.close` | `((self (& (chan T))))` | `unit` |

## Tests

`sync_mutex`, `sync_chan`, `sync_chan_thread`
