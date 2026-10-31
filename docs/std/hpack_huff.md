# std.hpack_huff

HPACK Huffman strings. Each symbol is an 8-bit prefix code, padded with one-bits.

Source: `std/hpack_huff.llr`. Import with `(use std.hpack_huff)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `hpack_huff.encode` | `((s (& str)))` | `str` |
| `hpack_huff.decode` | `((s (& str)))` | `(res str str)` |

## Tests

`hpack_huff_round`
