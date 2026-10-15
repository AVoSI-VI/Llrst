# std.os

Process environment.

Source: `std/os.llr`. Import with `(use std.os)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `os.args` | `()` | `(vec str)` |
| `os.arg` | `((i i64))` | `(opt str)` |
| `os.env` | `((name (& str)))` | `(opt str)` |
| `os.exit` | `((code i64))` | `never` |
| `os.cwd` | `()` | `str` |
| `os.shell` | `((cmd (& str)))` | `i64` |
| `os.arch` | `()` | `(& str)` |
| `os.is_tty` | `((fd i64))` | `bool` |

## Tests

`os_args`
