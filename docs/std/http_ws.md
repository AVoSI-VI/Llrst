# std.http_ws

WebSocket frames. Lengths of 7, 16, and 64 bits. Messages may span continuations.

Source: `std/http_ws.llr`. Import with `(use std.http_ws)` for module functions; types and methods need no import.

## Types

- `(struct frame (pub (fin bool)) (pub (op u8)) (pub (payload str)) (pub (next i64)))`
- `(struct piece (pub (op u8)) (pub (done bool)) (pub (data str)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `http_ws.next` | `((buf (& str)))` | `(res frame str)` |
| `http_ws.feed` | `((acc (&mut str)) (op0 (&mut u8)) (fin bool) (op u8) (payload (& str)))` | `(res piece str)` |
| `http_ws.encode` | `((op u8) (payload (& str)))` | `str` |

## Tests

`http_ws_frame`, `http_ws_feed`
