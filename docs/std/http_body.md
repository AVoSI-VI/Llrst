# std.http_body

HTTP/1.1 bodies. Content-Length, or chunked transfer with trailers.

Source: `std/http_body.llr`. Import with `(use std.http_body)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `http_body.take` | `((text (& str)) (at i64) (hs (&mut (vec str))))` | `(res str str)` |
| `http_body.write_chunked` | `((body (& str)) (trailers (& (vec str))))` | `str` |

## Tests

`http_body_len`, `http_body_chunk`
