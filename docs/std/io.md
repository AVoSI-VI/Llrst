# std.io

Standard streams. `io.read_fd` starts at 64 bytes and doubles after each full read until `read` returns the end.

Source: `std/io.llr`. Import with `(use std.io)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `io.put` | `((s (& str)))` | `unit` |
| `io.puts` | `((s (& str)))` | `unit` |
| `io.eput` | `((s (& str)))` | `unit` |
| `io.eputs` | `((s (& str)))` | `unit` |
| `io.read_fd` | `((fd i32))` | `str` |
| `io.stdin` | `()` | `str` |
| `io.read_line` | `((fd i32))` | `(opt str)` |
| `io.lines` | `((fd i32))` | `(vec str)` |
| `io.read_exact` | `((fd i32) (n i64))` | `str` |

## Tests

`io_streams`, `io_lines`, `io_exact`
