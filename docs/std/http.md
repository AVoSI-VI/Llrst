# std.http

One HTTP/1.1 request or response in a complete buffer. The path keeps any `?query`. Each header is one `name: value` string. `Content-Length` sets the body size; without it the body is empty. No chunked transfer and no trailers.

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

## Tests

`http_req`, `http_resp`
