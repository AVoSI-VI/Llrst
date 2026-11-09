# std.scan

Cursor over a byte string. Parsers share peek, names, and hex digits.

Source: `std/scan.llr`. Import with `(use std.scan)` for module functions; types and methods need no import.

## Types

- `(struct cur (pub (s (& str))) (pub (i i64)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `scan.peek` | `((c (& cur)))` | `u8` |
| `scan.at` | `((c (& cur)))` | `i64` |
| `scan.skip_ws` | `((c (&mut cur)))` | `unit` |
| `scan.is_name` | `((b u8))` | `bool` |
| `scan.read_name` | `((c (&mut cur)))` | `str` |
| `scan.hex_dig` | `((b u8))` | `(opt i64)` |
| `scan.expect` | `((c (&mut cur)) (b u8))` | `bool` |

## Tests

`scan_cur`
