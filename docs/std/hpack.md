# std.hpack

HPACK. Static table, dynamic table, Huffman strings, and size updates.

Source: `std/hpack.llr`. Import with `(use std.hpack)` for module functions; types and methods need no import.

## Types

- `(struct dyn (names (vec str)) (values (vec str)) (cap i64) (size i64))`
- `(struct hdrs (pub (names (vec str))) (pub (values (vec str))))`

## Functions

| name | params | returns |
|------|--------|---------|
| `hpack.empty` | `()` | `dyn` |
| `hpack.decode` | `((buf (& str)) (t (&mut dyn)))` | `(res hdrs str)` |
| `hpack.lit` | `((out (&mut str)) (name (& str)) (value (& str)) (huff bool))` | `unit` |

## Tests

`hpack_block`
