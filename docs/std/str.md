# std.str

Owned byte string `str`. Literals are `(& str)`.

Source: `std/str.llr`. Import with `(use std.str)` for module functions; types and methods need no import.

## Types

- `(struct str (p (ptr u8)) (len i64) (cap i64))`

## Functions

| name | params | returns |
|------|--------|---------|
| `str.new` | `()` | `str` |
| `str.drop` | `((self (&mut str)))` | `unit` |
| `str.len` | `((self (& str)))` | `i64` |
| `str.at` | `((self (& str)) (i i64))` | `u8` |
| `str.reserve` | `((self (&mut str)) (extra i64))` | `unit` |
| `str.push_byte` | `((self (&mut str)) (b u8))` | `unit` |
| `str.push` | `((self (&mut str)) (t (& str)))` | `unit` |
| `str.from` | `((t (& str)))` | `str` |
| `str.clone` | `((self (& str)))` | `str` |
| `str.cat` | `((a (& str)) (b (& str)))` | `str` |
| `str.eq` | `((a (& str)) (b (& str)))` | `bool` |
| `str.cmp` | `((a (& str)) (b (& str)))` | `i64` |
| `str.slice` | `((self (& str)) (lo i64) (hi i64))` | `str` |
| `str.find_from` | `((self (& str)) (needle (& str)) (start i64))` | `(opt i64)` |
| `str.find` | `((self (& str)) (needle (& str)))` | `(opt i64)` |
| `str.find_byte` | `((self (& str)) (b u8))` | `(opt i64)` |
| `str.has_byte` | `((self (& str)) (b u8))` | `bool` |
| `str.has` | `((self (& str)) (needle (& str)))` | `bool` |
| `str.starts` | `((self (& str)) (p (& str)))` | `bool` |
| `str.ends` | `((self (& str)) (p (& str)))` | `bool` |
| `str.split` | `((self (& str)) (sep u8))` | `(vec str)` |
| `str.lines` | `((self (& str)))` | `(vec str)` |
| `str.is_space` | `((b u8))` | `bool` |
| `str.trim` | `((self (& str)))` | `str` |
| `str.from_int` | `((n i64))` | `str` |
| `str.to_int` | `((self (& str)))` | `(opt i64)` |
| `str.replace` | `((self (& str)) (old (& str)) (new (& str)))` | `str` |
| `str.count` | `((self (& str)) (needle (& str)))` | `i64` |
| `str.join` | `((parts (& (vec str))) (sep (& str)))` | `str` |
| `str.lower` | `((self (& str)))` | `str` |
| `str.repeat` | `((self (& str)) (n i64))` | `str` |
| `str.cstr` | `((self (& str)))` | `str` |
| `str.from_raw` | `((p (ptr u8)) (n i64))` | `str` |
| `str.from_cstr` | `((p (ptr u8)))` | `str` |

## Tests

`str_basic`, `str_find_replace`, `str_int`, `str_lines_trim`
