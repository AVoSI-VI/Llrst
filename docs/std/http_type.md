# std.http_type

Content-Type from a file suffix. Unknown suffixes stay text/plain.

Source: `std/http_type.llr`. Import with `(use std.http_type)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `http_type.mime` | `((path (& str)))` | `str` |

## Tests

`http_type_mime`
