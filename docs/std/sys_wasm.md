# std.sys_wasm

WASI preview1, appended to std.sys for a wasm32 target. os_id is 3.

Source: `std/sys_wasm.llr`. Import with `(use std.sys_wasm)` for module functions; types and methods need no import.

## Constants

| name | type | value |
|------|------|-------|
| `sys_wasm.os_id` | `i64` | `3` |

## Functions

| name | params | returns |
|------|--------|---------|
| `sys_wasm.malloc` | `((n u64))` | `(ptr u8)` |
| `sys_wasm.realloc` | `((p (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_wasm.free` | `((p (ptr u8)))` | `unit` |
| `sys_wasm.write` | `((fd i32) (buf (ptr u8)) (n u64))` | `i64` |
| `sys_wasm.read` | `((fd i32) (buf (ptr u8)) (n u64))` | `i64` |
| `sys_wasm.memcpy` | `((d (ptr u8)) (s (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_wasm.memmove` | `((d (ptr u8)) (s (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_wasm.memcmp` | `((a (ptr u8)) (b (ptr u8)) (n u64))` | `i32` |
| `sys_wasm.strlen` | `((s (ptr u8)))` | `u64` |
| `sys_wasm.exit` | `((code i32))` | `never` |
| `sys_wasm._exit` | `((code i32))` | `never` |
| `sys_wasm.open` | `((path (ptr u8)) (flags i32) (mode i32))` | `i32` |
| `sys_wasm.close` | `((fd i32))` | `i32` |
| `sys_wasm.llr_boot` | `()` | `unit` |
| `sys_wasm.getenv` | `((name (ptr u8)))` | `(ptr u8)` |
| `sys_wasm.llr_trace_push` | `((slot (ptr u8)))` | `unit` |
| `sys_wasm.llr_trace_pop` | `((slot (ptr u8)))` | `unit` |
| `sys_wasm.llr_panic_frames` | `()` | `unit` |
| `sys_wasm.fork` | `()` | `i32` |
| `sys_wasm.waitpid` | `((pid i32) (status (ptr i32)) (opts i32))` | `i32` |
| `sys_wasm.getcwd` | `((buf (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys_wasm.opendir` | `((path (ptr u8)))` | `(ptr u8)` |
| `sys_wasm.readdir` | `((d (ptr u8)))` | `(ptr u8)` |
| `sys_wasm.closedir` | `((d (ptr u8)))` | `i32` |
| `sys_wasm.llr_readdir_name` | `((ent (ptr u8)))` | `(ptr u8)` |
| `sys_wasm.llr_stat_mtime` | `((path (ptr u8)) (out (ptr i64)))` | `i64` |
| `sys_wasm.llr_stat_size` | `((path (ptr u8)) (out (ptr i64)))` | `i64` |
| `sys_wasm.llr_now_ms` | `()` | `i64` |
| `sys_wasm.unlink` | `((path (ptr u8)))` | `i32` |
| `sys_wasm.mkdir` | `((path (ptr u8)) (mode i32))` | `i32` |
| `sys_wasm.rmdir` | `((path (ptr u8)))` | `i32` |
| `sys_wasm.rename` | `((a (ptr u8)) (b (ptr u8)))` | `i32` |
| `sys_wasm.system` | `((cmd (ptr u8)))` | `i32` |
| `sys_wasm.popen` | `((cmd (ptr u8)) (mode (ptr u8)))` | `(ptr u8)` |
| `sys_wasm.fileno` | `((f (ptr u8)))` | `i32` |
| `sys_wasm.pclose` | `((f (ptr u8)))` | `i32` |
| `sys_wasm.isatty` | `((fd i32))` | `i32` |
| `sys_wasm.socket` | `((domain i32) (ty i32) (proto i32))` | `i32` |
| `sys_wasm.bind` | `((fd i32) (addr (ptr u8)) (len u32))` | `i32` |
| `sys_wasm.listen` | `((fd i32) (backlog i32))` | `i32` |
| `sys_wasm.accept` | `((fd i32) (addr (ptr u8)) (len (ptr u32)))` | `i32` |
| `sys_wasm.connect` | `((fd i32) (addr (ptr u8)) (len u32))` | `i32` |
| `sys_wasm.getsockname` | `((fd i32) (addr (ptr u8)) (len (ptr u32)))` | `i32` |
| `sys_wasm.send` | `((fd i32) (buf (ptr u8)) (n u64) (flags i32))` | `i64` |
| `sys_wasm.recv` | `((fd i32) (buf (ptr u8)) (n u64) (flags i32))` | `i64` |
| `sys_wasm.poll` | `((fds (ptr u8)) (n u64) (timeout i32))` | `i32` |
| `sys_wasm.fcntl` | `((fd i32) (cmd i32) (arg i32))` | `i32` |
| `sys_wasm.__errno_location` | `()` | `(ptr i32)` |
| `sys_wasm.pthread_create` | `((thread (ptr u8)) (attr (ptr u8)) (start (fn ((ptr u8)) (ptr u8))) (arg (ptr u8)))` | `i32` |
| `sys_wasm.pthread_join` | `((thread u64) (ret (ptr u8)))` | `i32` |
| `sys_wasm.pthread_mutex_init` | `((m (ptr u8)) (attr (ptr u8)))` | `i32` |
| `sys_wasm.pthread_mutex_lock` | `((m (ptr u8)))` | `i32` |
| `sys_wasm.pthread_mutex_unlock` | `((m (ptr u8)))` | `i32` |
| `sys_wasm.pthread_mutex_destroy` | `((m (ptr u8)))` | `i32` |
| `sys_wasm.pthread_cond_init` | `((c (ptr u8)) (attr (ptr u8)))` | `i32` |
| `sys_wasm.pthread_cond_wait` | `((c (ptr u8)) (m (ptr u8)))` | `i32` |
| `sys_wasm.pthread_cond_broadcast` | `((c (ptr u8)))` | `i32` |
| `sys_wasm.pthread_cond_destroy` | `((c (ptr u8)))` | `i32` |
| `sys_wasm.setrlimit` | `((resource i32) (lim (ptr u8)))` | `i32` |
| `sys_wasm.llr_call_child` | `((f (fn (i64) i32)) (arg i64))` | `i32` |
| `sys_wasm.llr_spawn_self` | `((arg (ptr u8)))` | `i32` |
| `sys_wasm.llr_exec` | `((file (ptr u8)) (args (ptr u8)) (n u64))` | `i32` |
| `sys_wasm.llr_exe_path` | `((buf (ptr u8)) (n u64))` | `i64` |
| `sys_wasm.llr_mono_ms` | `()` | `i64` |
| `sys_wasm.llr_sleep_ms` | `((ms i64))` | `unit` |
| `sys_wasm.llr_entropy` | `((buf (ptr u8)) (n u64))` | `i64` |
| `sys_wasm.llr_set_env` | `((name (ptr u8)) (value (ptr u8)))` | `i32` |
| `sys_wasm.llr_pid` | `()` | `i64` |
| `sys_wasm.llr_io_get` | `()` | `(ptr u8)` |
| `sys_wasm.llr_io_set` | `((p (ptr u8)))` | `unit` |
| `sys_wasm.llr_run` | `((file (ptr u8)) (args (ptr u8)) (n u64) (in (ptr u8)) (in_n u64) (out (ptr u8)) (out_n (ptr u64)) (err (ptr u8)) (err_n (ptr u64)))` | `i32` |
| `sys_wasm.llr_sendto` | `((fd i32) (buf (ptr u8)) (n u64) (addr (ptr u8)) (len u32))` | `i64` |
| `sys_wasm.llr_recvfrom` | `((fd i32) (buf (ptr u8)) (n u64) (addr (ptr u8)) (alen (ptr u32)))` | `i64` |
| `sys_wasm.llr_set_timeout` | `((fd i32) (ms i64))` | `i32` |
| `sys_wasm.llr_resolve` | `((host (ptr u8)) (buf (ptr u8)) (cap u64) (out_n (ptr i64)))` | `i32` |
