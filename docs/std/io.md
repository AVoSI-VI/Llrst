# std.io

Standard streams.

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
