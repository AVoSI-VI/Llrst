# std.alloc

Free-list allocator. The heap header is four i64s: base, brk, end, free.

Source: `std/alloc.llr`. Import with `(use std.alloc)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `alloc.malloc` | `((h (ptr u8)) (n u64) (grow (fn (i32) i32)))` | `(ptr u8)` |
| `alloc.free` | `((h (ptr u8)) (p (ptr u8)))` | `unit` |
| `alloc.realloc` | `((h (ptr u8)) (p (ptr u8)) (n u64) (grow (fn (i32) i32)))` | `(ptr u8)` |

## Tests

`alloc_fit`
