# std.bytes

Little-endian and big-endian integers, LEB128, and CRC-32 on byte vectors.

Source: `std/bytes.llr`. Import with `(use std.bytes)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `bytes.from_str` | `((s (& str)))` | `(vec u8)` |
| `bytes.to_str` | `((b (& (vec u8))))` | `str` |
| `bytes.rd_le` | `((b (& (vec u8))) (at i64) (n i64))` | `i64` |
| `bytes.rd_be` | `((b (& (vec u8))) (at i64) (n i64))` | `i64` |
| `bytes.wr_le` | `((b (&mut (vec u8))) (v i64) (n i64))` | `unit` |
| `bytes.wr_be` | `((b (&mut (vec u8))) (v i64) (n i64))` | `unit` |
| `bytes.uvar` | `((b (&mut (vec u8))) (n i64))` | `unit` |
| `bytes.svar` | `((b (&mut (vec u8))) (n i64))` | `unit` |
| `bytes.rd_uvar` | `((b (& (vec u8))) (at (&mut i64)))` | `(res i64 str)` |
| `bytes.rd_svar` | `((b (& (vec u8))) (at (&mut i64)))` | `(res i64 str)` |
| `bytes.crc32` | `((b (& (vec u8))))` | `i64` |

## Tests

`bytes_int`
