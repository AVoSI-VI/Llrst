# std.http

One HTTP/1.1 request or response in a complete buffer, and `http.serve` on one poll loop. The path keeps any `?query`. A route matches method and path exactly. Each header is one `name: value` string. `Content-Length` sets the body size; without it the body is empty. No chunked transfer and no trailers. A `GET` with `Upgrade: websocket`, `Connection: Upgrade`, and `Sec-WebSocket-Version: 13` becomes a WebSocket and does not count toward `stop_after`. A text frame under 126 bytes is echoed. The poll table holds at most 4095 clients.

Source: `std/http.llr`. Import with `(use std.http)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `http.parse_req` | `((text (& str)))` | `(res req str)` |
| `http.parse_resp` | `((text (& str)))` | `(res resp str)` |
| `http.show_req` | `((r (& req)))` | `str` |
| `http.show_resp` | `((r (& resp)))` | `str` |
| `http.respond` | `((code i64) (reason (& str)) (ty (& str)) (body (& str)))` | `resp` |
| `http.header` | `((hs (& (vec str))) (name (& str)))` | `(opt str)` |
| `http.serve` | `((fd i64) (root (& str)) (routes (& (vec route))) (stop_after i64))` | `(res i64 str)` |

## Tests

`http_req`, `http_resp`, `http_file`, `http_many`, `http_ws`
