# std.http_route

Route paths. An exact path, a trailing star, or {name} segments.

Source: `std/http_route.llr`. Import with `(use std.http_route)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `http_route.hit` | `((pat (& str)) (path (& str)) (out (&mut (vec str))))` | `bool` |

## Tests

`http_route_hit`
