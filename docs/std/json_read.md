# std.json_read

JSON reader for objects, arrays, strings, ints, bools, and null.

Source: `std/json_read.llr`. Import with `(use std.json_read)` for module functions; types and methods need no import.

## Types

- `(struct jval (pub (kind u8)) (pub (text str)) (pub (items (vec jval))))`

## Functions

| name | params | returns |
|------|--------|---------|
| `json_read.read` | `((s (& str)))` | `(res jval str)` |
| `json_read.get` | `((v (& jval)) (key (& str)))` | `(opt i64)` |

## Tests

`json_read_vals`
