# std.sys

libc externs and the panic runtime. Not for programs; std only. `sys.os_id` is `1` on POSIX and `2` on Windows, defined in the host file.

Source: `std/sys.llr`, with externs from `std/sys_posix.llr` or `std/sys_windows.llr`. Import with `(use std.sys)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `sys.write_all` | `((fd i32) (p (ptr u8)) (n i64))` | `unit` |
| `sys.write_str` | `((fd i32) (s (& str)))` | `unit` |
| `sys.alloc` | `((n i64))` | `(ptr u8)` |
| `sys.grow` | `((p (ptr u8)) (n i64))` | `(ptr u8)` |
| `sys.checked_div` | `((loc (& str)) (a i64) (b i64) (rem bool))` | `i64` |
| `sys.abort` | `((loc (& str)) (msg (& str)))` | `never` |
| `sys.panic` | `((loc (& str)) (msg (& str)))` | `never` |

## Tests

`sys_mem`, `checked_div`
