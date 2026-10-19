# std.net

One blocking IPv4 connection on 127.0.0.1. `listen` uses backlog 1. Port 0 lets the kernel pick the port. `recv` returns one read; an empty string is end of stream. `send` writes every byte. Errors are `res.err` strings like `io.listen|port=`.

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

## Tests

`net_one`
