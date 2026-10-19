# std.sys

libc externs and the panic runtime. Not for programs; std only.

Source: `std/sys.llr`, with externs from `std/sys_posix.llr` or `std/sys_windows.llr`. Import with `(use std.sys)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `sys.write` | `((fd i32) (buf (ptr u8)) (n u64))` | `i64` |
| `sys.read` | `((fd i32) (buf (ptr u8)) (n u64))` | `i64` |
| `sys.open` | `((path (ptr u8)) (flags i32) (mode i32))` | `i32` |
| `sys.close` | `((fd i32))` | `i32` |
| `sys.malloc` | `((n u64))` | `(ptr u8)` |
| `sys.realloc` | `((p (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys.free` | `((p (ptr u8)))` | `unit` |
| `sys.memcpy` | `((d (ptr u8)) (s (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys.memmove` | `((d (ptr u8)) (s (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys.memcmp` | `((a (ptr u8)) (b (ptr u8)) (n u64))` | `i32` |
| `sys.exit` | `((code i32))` | `never` |
| `sys._exit` | `((code i32))` | `never` |
| `sys.fork` | `()` | `i32` |
| `sys.waitpid` | `((pid i32) (status (ptr i32)) (opts i32))` | `i32` |
| `sys.getenv` | `((name (ptr u8)))` | `(ptr u8)` |
| `sys.getcwd` | `((buf (ptr u8)) (n u64))` | `(ptr u8)` |
| `sys.strlen` | `((s (ptr u8)))` | `u64` |
| `sys.opendir` | `((path (ptr u8)))` | `(ptr u8)` |
| `sys.readdir` | `((d (ptr u8)))` | `(ptr u8)` |
| `sys.closedir` | `((d (ptr u8)))` | `i32` |
| `sys.llr_stat_mtime` | `((path (ptr u8)) (out (ptr i64)))` | `i64` |
| `sys.llr_stat_size` | `((path (ptr u8)) (out (ptr i64)))` | `i64` |
| `sys.llr_readdir_name` | `((ent (ptr u8)))` | `(ptr u8)` |
| `sys.llr_panic_frames` | `()` | `unit` |
| `sys.unlink` | `((path (ptr u8)))` | `i32` |
| `sys.mkdir` | `((path (ptr u8)) (mode i32))` | `i32` |
| `sys.rename` | `((a (ptr u8)) (b (ptr u8)))` | `i32` |
| `sys.system` | `((cmd (ptr u8)))` | `i32` |
| `sys.popen` | `((cmd (ptr u8)) (mode (ptr u8)))` | `(ptr u8)` |
| `sys.fileno` | `((f (ptr u8)))` | `i32` |
| `sys.pclose` | `((f (ptr u8)))` | `i32` |
| `sys.isatty` | `((fd i32))` | `i32` |
| `sys.socket` | `((domain i32) (ty i32) (proto i32))` | `i32` |
| `sys.bind` | `((fd i32) (addr (ptr u8)) (len u32))` | `i32` |
| `sys.listen` | `((fd i32) (backlog i32))` | `i32` |
| `sys.accept` | `((fd i32) (addr (ptr u8)) (len (ptr u32)))` | `i32` |
| `sys.connect` | `((fd i32) (addr (ptr u8)) (len u32))` | `i32` |
| `sys.getsockname` | `((fd i32) (addr (ptr u8)) (len (ptr u32)))` | `i32` |
| `sys.send` | `((fd i32) (buf (ptr u8)) (n u64) (flags i32))` | `i64` |
| `sys.recv` | `((fd i32) (buf (ptr u8)) (n u64) (flags i32))` | `i64` |
| `sys.poll` | `((fds (ptr u8)) (n u64) (timeout i32))` | `i32` |
| `sys.fcntl` | `((fd i32) (cmd i32) (arg i32))` | `i32` |
| `sys.__errno_location` | `()` | `(ptr i32)` |
| `sys.pthread_create` | `((thread (ptr u8)) (attr (ptr u8)) (start (fn ((ptr u8)) (ptr u8))) (arg (ptr u8)))` | `i32` |
| `sys.pthread_join` | `((thread u64) (ret (ptr u8)))` | `i32` |
| `sys.pthread_mutex_init` | `((m (ptr u8)) (attr (ptr u8)))` | `i32` |
| `sys.pthread_mutex_lock` | `((m (ptr u8)))` | `i32` |
| `sys.pthread_mutex_unlock` | `((m (ptr u8)))` | `i32` |
| `sys.pthread_mutex_destroy` | `((m (ptr u8)))` | `i32` |
| `sys.pthread_cond_init` | `((c (ptr u8)) (attr (ptr u8)))` | `i32` |
| `sys.pthread_cond_wait` | `((c (ptr u8)) (m (ptr u8)))` | `i32` |
| `sys.pthread_cond_broadcast` | `((c (ptr u8)))` | `i32` |
| `sys.pthread_cond_destroy` | `((c (ptr u8)))` | `i32` |
| `sys.write_all` | `((fd i32) (p (ptr u8)) (n i64))` | `unit` |
| `sys.write_str` | `((fd i32) (s (& str)))` | `unit` |
| `sys.alloc` | `((n i64))` | `(ptr u8)` |
| `sys.grow` | `((p (ptr u8)) (n i64))` | `(ptr u8)` |
| `sys.checked_div` | `((loc (& str)) (a i64) (b i64) (rem bool))` | `i64` |
| `sys.abort` | `((loc (& str)) (msg (& str)))` | `never` |
| `sys.panic` | `((loc (& str)) (msg (& str)))` | `never` |

## Tests

`checked_div`
