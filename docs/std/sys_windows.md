# std.sys_windows

Windows externs with the same link names, appended to std.sys for a Windows target. Not linked here.

Source: `std/sys_windows.llr`. Import with `(use std.sys_windows)` for module functions; types and methods need no import.

## Constants

| name | type | value |
|------|------|-------|
| `sys_windows.os_id` | `i64` | `2` |

## Functions

| name | params | returns |
|------|--------|---------|
| `sys_windows.write` | `((fd i32) (buf (ptr u8)) (n u64))` | `i64` |
| `sys_windows.read` | `((fd i32) (buf (ptr u8)) (n u64))` | `i64` |
| `sys_windows.open` | `((path (ptr u8)) (flags i32) (mode i32))` | `i32` |
| `sys_windows.close` | `((fd i32))` | `i32` |
| `sys_windows.malloc` | `((n u64))` | `(ptr u8)` |
| `sys_windows.realloc` | `((p (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_windows.free` | `((p (ptr u8)))` | `unit` |
| `sys_windows.memcpy` | `((d (ptr u8)) (s (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_windows.memmove` | `((d (ptr u8)) (s (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_windows.memcmp` | `((a (ptr u8)) (b (ptr u8)) (n u64))` | `i32` |
| `sys_windows.exit` | `((code i32))` | `never` |
| `sys_windows._exit` | `((code i32))` | `never` |
| `sys_windows.fork` | `()` | `i32` |
| `sys_windows.waitpid` | `((pid i32) (status (ptr i32)) (opts i32))` | `i32` |
| `sys_windows.getenv` | `((name (ptr u8)))` | `(ptr u8)` |
| `sys_windows.getcwd` | `((buf (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_windows.strlen` | `((s (ptr u8)))` | `u64` |
| `sys_windows.opendir` | `((path (ptr u8)))` | `(ptr u8)` |
| `sys_windows.readdir` | `((d (ptr u8)))` | `(ptr u8)` |
| `sys_windows.closedir` | `((d (ptr u8)))` | `i32` |
| `sys_windows.llr_stat_mtime` | `((path (ptr u8)) (out (ptr i64)))` | `i64` |
| `sys_windows.llr_stat_size` | `((path (ptr u8)) (out (ptr i64)))` | `i64` |
| `sys_windows.llr_readdir_name` | `((ent (ptr u8)))` | `(ptr u8)` |
| `sys_windows.llr_panic_frames` | `()` | `unit` |
| `sys_windows.unlink` | `((path (ptr u8)))` | `i32` |
| `sys_windows.mkdir` | `((path (ptr u8)) (mode i32))` | `i32` |
| `sys_windows.rmdir` | `((path (ptr u8)))` | `i32` |
| `sys_windows.rename` | `((a (ptr u8)) (b (ptr u8)))` | `i32` |
| `sys_windows.system` | `((cmd (ptr u8)))` | `i32` |
| `sys_windows.popen` | `((cmd (ptr u8)) (mode (ptr u8)))` | `(ptr u8)` |
| `sys_windows.fileno` | `((f (ptr u8)))` | `i32` |
| `sys_windows.pclose` | `((f (ptr u8)))` | `i32` |
| `sys_windows.isatty` | `((fd i32))` | `i32` |
| `sys_windows.socket` | `((domain i32) (ty i32) (proto i32))` | `i32` |
| `sys_windows.bind` | `((fd i32) (addr (ptr u8)) (len u32))` | `i32` |
| `sys_windows.listen` | `((fd i32) (backlog i32))` | `i32` |
| `sys_windows.accept` | `((fd i32) (addr (ptr u8)) (len (ptr u32)))` | `i32` |
| `sys_windows.connect` | `((fd i32) (addr (ptr u8)) (len u32))` | `i32` |
| `sys_windows.getsockname` | `((fd i32) (addr (ptr u8)) (len (ptr u32)))` | `i32` |
| `sys_windows.send` | `((fd i32) (buf (ptr u8)) (n u64) (flags i32))` | `i64` |
| `sys_windows.recv` | `((fd i32) (buf (ptr u8)) (n u64) (flags i32))` | `i64` |
| `sys_windows.poll` | `((fds (ptr u8)) (n u64) (timeout i32))` | `i32` |
| `sys_windows.fcntl` | `((fd i32) (cmd i32) (arg i32))` | `i32` |
| `sys_windows.__errno_location` | `()` | `(ptr i32)` |
| `sys_windows.pthread_create` | `((thread (ptr u8)) (attr (ptr u8)) (start (fn ((ptr u8)) (ptr u8))) (arg (ptr u8)))` | `i32` |
| `sys_windows.pthread_join` | `((thread u64) (ret (ptr u8)))` | `i32` |
| `sys_windows.pthread_mutex_init` | `((m (ptr u8)) (attr (ptr u8)))` | `i32` |
| `sys_windows.pthread_mutex_lock` | `((m (ptr u8)))` | `i32` |
| `sys_windows.pthread_mutex_unlock` | `((m (ptr u8)))` | `i32` |
| `sys_windows.pthread_mutex_destroy` | `((m (ptr u8)))` | `i32` |
| `sys_windows.pthread_cond_init` | `((c (ptr u8)) (attr (ptr u8)))` | `i32` |
| `sys_windows.pthread_cond_wait` | `((c (ptr u8)) (m (ptr u8)))` | `i32` |
| `sys_windows.pthread_cond_broadcast` | `((c (ptr u8)))` | `i32` |
| `sys_windows.pthread_cond_destroy` | `((c (ptr u8)))` | `i32` |
| `sys_windows.setrlimit` | `((resource i32) (lim (ptr u8)))` | `i32` |
| `sys_windows.llr_call_child` | `((f (fn (i64) i32)) (arg i64))` | `i32` |
| `sys_windows.llr_spawn_self` | `((arg (ptr u8)))` | `i32` |
| `sys_windows.llr_exe_path` | `((buf (ptr u8)) (n u64))` | `i64` |
