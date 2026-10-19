# std.net

IPv4 on 127.0.0.1. `listen` uses backlog 1 and blocks in `accept`. `poll_serve` sets that fd nonblocking, raises the backlog to 128, and reads many connections on the calling thread. The poll table holds 4096 fds. The ready callback is a function pointer plus a context pointer. `recv` returns one read; an empty string is end of stream. `send` writes every byte. Errors are `res.err` strings like `io.listen|port=` and `io.poll|fd=`.

Source: `std/net.llr`. Import with `(use std.net)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `net.listen` | `((port i64))` | `(res i64 str)` |
| `net.local_port` | `((fd i64))` | `(res i64 str)` |
| `net.accept` | `((fd i64))` | `(res i64 str)` |
| `net.connect` | `((port i64))` | `(res i64 str)` |
| `net.recv` | `((fd i64) (n i64))` | `(res str str)` |
| `net.send` | `((fd i64) (data (& str)))` | `(res unit str)` |
| `net.close` | `((fd i64))` | `unit` |
| `net.poll_serve` | `((listen_fd i64) (max i64) (each (fn (i64 (& str) (ptr u8)) bool)) (ctx (ptr u8)))` | `(res i64 str)` |

## Tests

`net_one`, `net_poll`
