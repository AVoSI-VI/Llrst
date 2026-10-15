# std.path

Path string helpers; `/` only.

Source: `std/path.llr`. Import with `(use std.path)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `path.join` | `((a (& str)) (b (& str)))` | `str` |
| `path.base` | `((p (& str)))` | `str` |
| `path.dir` | `((p (& str)))` | `str` |
| `path.ext` | `((p (& str)))` | `str` |
| `path.stem` | `((p (& str)))` | `str` |
| `path.is_abs` | `((p (& str)))` | `bool` |
| `path.clean` | `((p (& str)))` | `str` |

## Tests

`path_parts`
