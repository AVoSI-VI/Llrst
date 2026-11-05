# std.url

Percent-encoding (RFC 3986 unreserved set) and a small URL split.

Source: `std/url.llr`. Import with `(use std.url)` for module functions; types and methods need no import.

## Types

- `(struct url (pub (scheme str)) (pub (host str)) (pub (port i64)) (pub (path str)) (pub (query str)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `url.encode` | `((s (& str)))` | `str` |
| `url.decode` | `((s (& str)))` | `(opt str)` |
| `url.parse` | `((s (& str)))` | `(opt url)` |

## Tests

`url_parts`
