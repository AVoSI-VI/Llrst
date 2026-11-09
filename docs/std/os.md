# std.os

Process environment. `os.cwd` starts at 1 byte and doubles until `getcwd` succeeds. Failure, including a path past 1 MiB, is `res.err` of `io.cwd`.

Source: `std/os.llr`. Import with `(use std.os)` for module functions; types and methods need no import.

## Types

- `(struct cmd (pub (out str)) (pub (code i64)) (pub (err str)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `os.args` | `()` | `(vec str)` |
| `os.arg` | `((i i64))` | `(opt str)` |
| `os.set_env` | `((name (& str)) (value (& str)))` | `bool` |
| `os.home` | `()` | `(opt str)` |
| `os.pid` | `()` | `i64` |
| `os.env` | `((name (& str)))` | `(opt str)` |
| `os.exit` | `((code i64))` | `never` |
| `os.cwd` | `()` | `(res str str)` |
| `os.shell` | `((cmd (& str)))` | `i64` |
| `os.capture` | `((command (& str)))` | `cmd` |
| `os.arch` | `()` | `(& str)` |
| `os.is_tty` | `((fd i64))` | `bool` |
| `os.exec` | `((file (& str)) (args (& (vec str))))` | `i64` |
| `os.run` | `((file (& str)) (args (& (vec str))) (stdin (& str)))` | `(res cmd str)` |
| `os.exe_path` | `()` | `str` |

## Tests

`os_cwd`, `os_args`, `os_env`, `os_run`, `os_capture`, `os_exec_args`, `os_exec`
