# std.fs

Files and directories. Errors are `res.err` strings like `io.read|path=x`.
`fs.mtime` and `fs.size` call `runtime/llr_stat.c`, which reads `st_mtime` and `st_size`
through the host C compiler. `fs.list` takes each name from `llr_readdir_name`.

Source: `std/fs.llr`. Import with `(use std.fs)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `fs.read` | `((path (& str)))` | `(res str str)` |
| `fs.write` | `((path (& str)) (data (& str)))` | `(res unit str)` |
| `fs.append` | `((path (& str)) (data (& str)))` | `(res unit str)` |
| `fs.exists` | `((path (& str)))` | `bool` |
| `fs.is_dir` | `((path (& str)))` | `bool` |
| `fs.remove` | `((path (& str)))` | `(res unit str)` |
| `fs.mkdir` | `((path (& str)))` | `(res unit str)` |
| `fs.rename` | `((from (& str)) (to (& str)))` | `(res unit str)` |
| `fs.list` | `((path (& str)))` | `(res (vec str) str)` |
| `fs.mtime` | `((path (& str)))` | `(res i64 str)` |
| `fs.size` | `((path (& str)))` | `(res i64 str)` |
| `fs.mkdir_all` | `((path (& str)))` | `(res unit str)` |
| `fs.rmdir` | `((path (& str)))` | `(res unit str)` |
| `fs.remove_all` | `((path (& str)))` | `(res unit str)` |

## Tests

`fs_edit`, `fs_roundtrip`, `fs_list`, `fs_tree`, `fs_read_grow`
