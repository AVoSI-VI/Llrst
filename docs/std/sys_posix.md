# std.sys_posix

POSIX libc externs, appended to std.sys on every non-Windows target.

Source: `std/sys_posix.llr`. Import with `(use std.sys_posix)` for module functions; types and methods need no import.

## Constants

| name | type | value |
|------|------|-------|
| `sys_posix.os_id` | `i64` | `1` |

## Functions

| name | params | returns |
|------|--------|---------|
| `sys_posix.write` | `((fd i32) (buf (ptr u8)) (n u64))` | `i64` |
| `sys_posix.read` | `((fd i32) (buf (ptr u8)) (n u64))` | `i64` |
| `sys_posix.open` | `((path (ptr u8)) (flags i32) (mode i32))` | `i32` |
| `sys_posix.close` | `((fd i32))` | `i32` |
| `sys_posix.malloc` | `((n u64))` | `(ptr u8)` |
| `sys_posix.realloc` | `((p (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_posix.free` | `((p (ptr u8)))` | `unit` |
| `sys_posix.memcpy` | `((d (ptr u8)) (s (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_posix.memmove` | `((d (ptr u8)) (s (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_posix.memcmp` | `((a (ptr u8)) (b (ptr u8)) (n u64))` | `i32` |
| `sys_posix.exit` | `((code i32))` | `never` |
| `sys_posix._exit` | `((code i32))` | `never` |
| `sys_posix.fork` | `()` | `i32` |
| `sys_posix.waitpid` | `((pid i32) (status (ptr i32)) (opts i32))` | `i32` |
| `sys_posix.getenv` | `((name (ptr u8)))` | `(ptr u8)` |
| `sys_posix.getcwd` | `((buf (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_posix.strlen` | `((s (ptr u8)))` | `u64` |
| `sys_posix.opendir` | `((path (ptr u8)))` | `(ptr u8)` |
| `sys_posix.readdir` | `((d (ptr u8)))` | `(ptr u8)` |
| `sys_posix.closedir` | `((d (ptr u8)))` | `i32` |
| `sys_posix.llr_stat_mtime` | `((path (ptr u8)) (out (ptr i64)))` | `i64` |
| `sys_posix.llr_stat_size` | `((path (ptr u8)) (out (ptr i64)))` | `i64` |
| `sys_posix.llr_readdir_name` | `((ent (ptr u8)))` | `(ptr u8)` |
| `sys_posix.llr_panic_frames` | `()` | `unit` |
| `sys_posix.unlink` | `((path (ptr u8)))` | `i32` |
| `sys_posix.mkdir` | `((path (ptr u8)) (mode i32))` | `i32` |
| `sys_posix.rename` | `((a (ptr u8)) (b (ptr u8)))` | `i32` |
| `sys_posix.system` | `((cmd (ptr u8)))` | `i32` |
| `sys_posix.popen` | `((cmd (ptr u8)) (mode (ptr u8)))` | `(ptr u8)` |
| `sys_posix.fileno` | `((f (ptr u8)))` | `i32` |
| `sys_posix.pclose` | `((f (ptr u8)))` | `i32` |
| `sys_posix.isatty` | `((fd i32))` | `i32` |
| `sys_posix.socket` | `((domain i32) (ty i32) (proto i32))` | `i32` |
| `sys_posix.bind` | `((fd i32) (addr (ptr u8)) (len u32))` | `i32` |
| `sys_posix.listen` | `((fd i32) (backlog i32))` | `i32` |
| `sys_posix.accept` | `((fd i32) (addr (ptr u8)) (len (ptr u32)))` | `i32` |
| `sys_posix.connect` | `((fd i32) (addr (ptr u8)) (len u32))` | `i32` |
| `sys_posix.getsockname` | `((fd i32) (addr (ptr u8)) (len (ptr u32)))` | `i32` |
| `sys_posix.send` | `((fd i32) (buf (ptr u8)) (n u64) (flags i32))` | `i64` |
| `sys_posix.recv` | `((fd i32) (buf (ptr u8)) (n u64) (flags i32))` | `i64` |
| `sys_posix.poll` | `((fds (ptr u8)) (n u64) (timeout i32))` | `i32` |
| `sys_posix.fcntl` | `((fd i32) (cmd i32) (arg i32))` | `i32` |
| `sys_posix.__errno_location` | `()` | `(ptr i32)` |
| `sys_posix.pthread_create` | `((thread (ptr u8)) (attr (ptr u8)) (start (fn ((ptr u8)) (ptr u8))) (arg (ptr u8)))` | `i32` |
| `sys_posix.pthread_join` | `((thread u64) (ret (ptr u8)))` | `i32` |
| `sys_posix.pthread_mutex_init` | `((m (ptr u8)) (attr (ptr u8)))` | `i32` |
| `sys_posix.pthread_mutex_lock` | `((m (ptr u8)))` | `i32` |
| `sys_posix.pthread_mutex_unlock` | `((m (ptr u8)))` | `i32` |
| `sys_posix.pthread_mutex_destroy` | `((m (ptr u8)))` | `i32` |
| `sys_posix.pthread_cond_init` | `((c (ptr u8)) (attr (ptr u8)))` | `i32` |
| `sys_posix.pthread_cond_wait` | `((c (ptr u8)) (m (ptr u8)))` | `i32` |
| `sys_posix.pthread_cond_broadcast` | `((c (ptr u8)))` | `i32` |
| `sys_posix.pthread_cond_destroy` | `((c (ptr u8)))` | `i32` |
| `sys_posix.setrlimit` | `((resource i32) (lim (ptr u8)))` | `i32` |
| `sys_posix.llr_call_child` | `((f (fn (i64) i32)) (arg i64))` | `i32` |
| `sys_posix.llr_spawn_self` | `((arg (ptr u8)))` | `i32` |
| `sys_posix.llr_exe_path` | `((buf (ptr u8)) (n u64))` | `i64` |
