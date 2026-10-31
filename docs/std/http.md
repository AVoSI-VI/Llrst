# std.http

HTTP/1.1 messages and `http.serve` on one poll loop. `Content-Length` or chunked transfer sets the body. A chunk extension is skipped. Trailers are appended and the last header of a name wins. After a chunked body is decoded, `Transfer-Encoding` is replaced by `Content-Length`. A route matches an exact path, a trailing `*`, or `{name}` segments. `(http.param)` reads a capture. A WebSocket message may span continuation frames and use 7-bit, 16-bit, or 64-bit lengths. Text and binary are echoed. Ping is answered with pong. A message over 1 MiB is `http.frame`. The client table is not capped.

Source: `std/http.llr`. Import with `(use std.http)` for module functions; types and methods need no import.

## Types

- `(struct req (pub (method str)) (pub (path str)) (pub (headers (vec str))) (pub (body str)) (pub (params (vec str))))`
- `(struct resp (pub (code i64)) (pub (reason str)) (pub (headers (vec str))) (pub (body str)))`
- `(struct route (pub (method str)) (pub (path str)) (pub (work (fn ((& req) (ptr u8)) resp))))`

## Functions

| name | params | returns |
|------|--------|---------|
| `http.header` | `((hs (& (vec str))) (name (& str)))` | `(opt str)` |
| `http.param` | `((hs (& (vec str))) (name (& str)))` | `(opt str)` |
| `http.parse_req` | `((text (& str)))` | `(res req str)` |
| `http.parse_resp` | `((text (& str)))` | `(res resp str)` |
| `http.show_req` | `((r (& req)))` | `str` |
| `http.show_resp` | `((r (& resp)))` | `str` |
| `http.respond` | `((code i64) (reason (& str)) (ty (& str)) (body (& str)))` | `resp` |
| `http.serve` | `((fd i64) (root (& str)) (routes (& (vec route))) (stop_after i64))` | `(res i64 str)` |

## Tests

`http_req`, `http_resp`, `http_file`, `http_many`, `http_ws`, `http_bits`
