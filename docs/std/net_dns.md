# std.net_dns

DNS names and IP literals. `fill` writes a Linux sockaddr of 16 or 28 bytes.

Source: `std/net_dns.llr`. Import with `(use std.net_dns)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `net_dns.fill` | `((host (& str)) (port i64) (p (ptr u8)))` | `i64` |
| `net_dns.resolve` | `((host (& str)))` | `(res (vec str) str)` |
| `net_dns.prep` | `((host (& str)) (port i64) (p (ptr u8)))` | `i64` |

## Tests

`net_dns_lit`, `net_dns_resolve`
