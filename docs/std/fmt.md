# std.fmt

Pipe-record formatting for model-facing output.

Source: `std/fmt.llr`. Import with `(use std.fmt)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `fmt.pipe2` | `((a (& str)) (b (& str)))` | `str` |
| `fmt.pipe3` | `((a (& str)) (b (& str)) (c (& str)))` | `str` |
| `fmt.pipe4` | `((a (& str)) (b (& str)) (c (& str)) (d (& str)))` | `str` |
| `fmt.pipe5` | `((a (& str)) (b (& str)) (c (& str)) (d (& str)) (e (& str)))` | `str` |
| `fmt.field` | `((s (& str)))` | `str` |
| `fmt.kv` | `((k (& str)) (v (& str)))` | `str` |
| `fmt.int` | `((n i64))` | `str` |
| `fmt.pad_left` | `((s (& str)) (width i64) (fill u8))` | `str` |

## Tests

`fmt_pipe`
