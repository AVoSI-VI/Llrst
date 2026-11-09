# std.str_more

More str methods. str.llr stays the core; this file is a second impl.

Source: `std/str_more.llr`. Import with `(use std.str_more)` for module functions; types and methods need no import.

## Types

- `(struct uch (pub (cp i64)) (pub (n i64)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `str.upper` | `((self (& str)))` | `str` |
| `str.trim_start` | `((self (& str)))` | `str` |
| `str.trim_end` | `((self (& str)))` | `str` |
| `str.rfind_byte` | `((self (& str)) (b u8))` | `(opt i64)` |
| `str.rfind` | `((self (& str)) (needle (& str)))` | `(opt i64)` |
| `str.split_str` | `((self (& str)) (sep (& str)))` | `(vec str)` |
| `str.replace_byte` | `((self (& str)) (old u8) (new u8))` | `str` |
| `str.pad_right` | `((self (& str)) (width i64) (fill u8))` | `str` |
| `str.to_bytes` | `((self (& str)))` | `(vec u8)` |
| `str.from_bytes` | `((v (& (vec u8))))` | `str` |
| `str.utf8_at` | `((self (& str)) (i i64))` | `uch` |
| `str.push_utf8` | `((self (&mut str)) (cp i64))` | `bool` |
| `str.utf8_ok` | `((self (& str)))` | `bool` |
| `str.to_int_radix` | `((self (& str)) (radix i64))` | `(opt i64)` |
| `str.from_int_radix` | `((n i64) (radix i64))` | `str` |
| `str.to_f64` | `((self (& str)))` | `(opt f64)` |
| `str.from_f64` | `((x f64) (digits i64))` | `str` |

## Tests

`str_text`, `str_bytes_utf`, `str_radix_float`, `str_more_helpers`
