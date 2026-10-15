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

Program output destined for a model is one record per line, `|` separated, as in
[../spec/errors.md](../spec/errors.md). Diagnostics go to stderr, data to stdout.
