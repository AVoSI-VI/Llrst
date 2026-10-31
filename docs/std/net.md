# std.net

IPv4. `listen` binds 127.0.0.1. `connect` takes a dotted quad and a port; a bad host is `io.connect|host=`. `poll_serve` reads many connections on the calling thread. `max < 0` does not cap the client table. The ready callback is a closure and can capture a binding. `recv` returns one read; an empty string is end of stream. `send` writes every byte.

Source: `std/net.llr`. Import with `(use std.net)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `net.listen` | `((port i64))` | `(res i64 str)` |
| `net.local_port` | `((fd i64))` | `(res i64 str)` |
| `net.accept` | `((fd i64))` | `(res i64 str)` |
| `net.connect` | `((host (& str)) (port i64))` | `(res i64 str)` |
| `net.recv` | `((fd i64) (n i64))` | `(res str str)` |
| `net.send` | `((fd i64) (data (& str)))` | `(res unit str)` |
| `net.close` | `((fd i64))` | `unit` |
| `net.poll_serve` | `((listen_fd i64) (max i64) (each (clos (i64 (& str)) bool)))` | `(res i64 str)` |

## Tests

`net_one`, `net_poll`, `net_helpers`
