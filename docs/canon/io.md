# Canon: io

| task | form |
|------|------|
| stdout line | `(io.puts (& s))` appends `\n` |
| stdout raw | `(io.put (& s))` |
| stderr line | `(io.eputs (& s))` |
| read all stdin | `(io.stdin)` returns `str` |
| read file | `(fs.read (& path))` returns `(res str str)` |
| write file | `(fs.write (& path) (& data))` returns `(res unit str)` |
| exists | `(fs.exists (& path))` |
| is dir | `(fs.is_dir (& path))` |
| list dir | `(fs.list (& path))` returns `(res (vec str) str)`, names only, sorted |
| mtime | `(fs.mtime (& path))` returns `(res i64 str)` seconds |
| join path | `(path.join (& a) (& b))` |
| base name | `(path.base (& p))` |
| dir name | `(path.dir (& p))` |
| extension | `(path.ext (& p))` without dot |
| args | `(os.args)` returns `(vec str)` including argv0 |
| env | `(os.env (& name))` returns `(opt str)` |
| exit | `(os.exit code)` |
| cwd | `(os.cwd)` |
| listen | `(net.listen port)` returns `(res i64 str)`, the listening fd. Port 0 is any port. Backlog 1 |
| local port | `(net.local_port fd)` returns `(res i64 str)` |
| accept | `(net.accept fd)` blocks and returns `(res i64 str)` |
| connect | `(net.connect port)` returns `(res i64 str)` to 127.0.0.1 |
| recv | `(net.recv fd n)` returns `(res str str)`. An empty string is end of stream |
| send | `(net.send fd (& data))` returns `(res unit str)` and writes every byte |
| close socket | `(net.close fd)` |
| poll serve | `(net.poll_serve fd max each ctx)` returns `(res i64 str)`, the number of reads. `each` is `(fn (i64 (& str) (ptr u8)) bool)` |

Program output destined for a model is one record per line, `|` separated, as in
[../spec/errors.md](../spec/errors.md). Diagnostics go to stderr, data to stdout.
