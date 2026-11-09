# Canon: io

| task | form |
|------|------|
| stdout line | `(io.puts (& s))` appends `\n` |
| stdout raw | `(io.put (& s))` |
| stderr line | `(io.eputs (& s))` |
| stderr raw | `(io.eput (& s))` |
| read all stdin | `(io.stdin)` returns `str` |
| read line | `(io.read_line fd)` returns `(opt str)`. The newline is not included. `none` is end of file. Bytes past the line stay with that fd |
| read lines | `(io.lines fd)` returns `(vec str)`. A final newline does not add an empty line |
| read exact | `(io.read_exact fd n)` returns `str` of at most `n` bytes. A short string is end of file |
| read file | `(fs.read (& path))` returns `(res str str)` |
| write file | `(fs.write (& path) (& data))` returns `(res unit str)` |
| append file | `(fs.append (& path) (& data))` returns `(res unit str)` |
| exists | `(fs.exists (& path))` |
| is file | `(fs.is_file (& path))` |
| copy file | `(fs.copy (& from) (& to))` returns `(res unit str)` from the read or the write |
| temp dir | `(fs.temp_dir)` returns `/tmp`. Windows maps that path to the user temp directory |
| absolute path | `(fs.abs (& path))` returns `(res str str)`, cleaned. A relative path joins the cwd |
| is dir | `(fs.is_dir (& path))` |
| remove file | `(fs.remove (& path))` returns `(res unit str)` |
| make dir | `(fs.mkdir (& path))` returns `(res unit str)` |
| make dir tree | `(fs.mkdir_all (& path))` returns `(res unit str)`. `.` and an existing directory are success |
| remove tree | `(fs.remove_all (& path))` returns `(res unit str)`. A file is unlinked; a directory is removed with its contents |
| rename | `(fs.rename (& from) (& to))` returns `(res unit str)` |
| list dir | `(fs.list (& path))` returns `(res (vec str) str)`, names only, sorted |
| mtime | `(fs.mtime (& path))` returns `(res i64 str)` seconds |
| size | `(fs.size (& path))` returns `(res i64 str)` bytes |
| separator | `(path.sep)` returns `/`, or `\` when `sys.os_id` is 2 |
| join path | `(path.join (& a) (& b))` inserts `/`. On Windows a `\` in either argument is a separator too |
| base name | `(path.base (& p))` |
| stem | `(path.stem (& p))` drops the extension |
| dir name | `(path.dir (& p))` |
| extension | `(path.ext (& p))` without dot |
| absolute | `(path.is_abs (& p))` |
| clean | `(path.clean (& p))` drops `.` and duplicate slashes, keeps `..`, and emits `/`. `C:\a\b` becomes `C:/a/b` |
| args | `(os.args)` returns `(vec str)` including argv0 |
| env | `(os.env (& name))` returns `(opt str)` |
| set env | `(os.set_env (& name) (& value))` returns `bool` |
| home | `(os.home)` returns `(opt str)`. `HOME`, or `USERPROFILE` on Windows |
| pid | `(os.pid)` returns `i64` |
| run | `(os.run (& file) (& args) (& stdin))` returns `(res cmd str)`. No shell. `cmd` has `out`, `code`, and `err`. Spawn failure is `io.spawn\|file=` |
| shell | `(os.shell (& cmd))` returns the exit code |
| capture | `(os.capture (& cmd))` returns `cmd` with stdout and the exit code. `err` is empty |
| arch | `(os.arch)` returns `(& str)` |
| tty | `(os.is_tty fd)` |
| exit | `(os.exit code)` |
| cwd | `(os.cwd)` returns `(res str str)`. Failure, including a path past 1 MiB, is `io.cwd` |
| listen | `(net.listen port)` returns `(res i64 str)`, the listening fd. Port 0 is any port. Binds 127.0.0.1 |
| listen6 | `(net.listen6 port)` is the same on `::1` |
| local port | `(net.local_port fd)` returns `(res i64 str)` |
| accept | `(net.accept fd)` blocks and returns `(res i64 str)` |
| connect | `(net.connect host port)` returns `(res i64 str)`. `host` is a dotted IPv4, an IPv6 literal, or a name |
| resolve | `(net_dns.resolve host)` returns `(res (vec str) str)`. Failure is `io.resolve\|host=` |
| udp bind | `(net.udp_bind port)` returns `(res i64 str)`, bound to 127.0.0.1 |
| udp send | `(net.udp_send fd host port (& data))` returns `(res unit str)` |
| udp recv | `(net.udp_recv fd n)` returns `(res str str)`. An empty string is an empty datagram |
| socket timeout | `(net.set_timeout fd ms)` sets send and receive. `ms <= 0` waits forever. Failure is `io.timeout\|fd=` |
| recv | `(net.recv fd n)` returns `(res str str)`. An empty string is end of stream |
| send | `(net.send fd (& data))` returns `(res unit str)` and writes every byte |
| close socket | `(net.close fd)` |
| poll serve | `(net.poll_serve fd max each)` returns `(res i64 str)`, the number of reads. `each` is `(clos (i64 (& str)) bool)`. `max < 0` does not cap clients |

## Time, bytes, and JSON

| task | form |
|------|------|
| wall clock | `(time.now_ms)` milliseconds since the Unix epoch. `(fs.now_ms)` forwards here for one release |
| monotonic clock | `(time.mono_ms)` milliseconds, `(time.mono_ns)` nanoseconds |
| sleep | `(time.sleep_ms n)` |
| UTC date | `(time.utc ms)` returns `date`. `(time.iso8601 (& d))` is `YYYY-MM-DDTHH:MM:SS.mmmZ`. `(time.parse_iso (& s))` reads that form |
| random | `(rand.seed s)`, `(rand.next (&mut r))`, `(rand.below (&mut r) n)`, `(rand.f64 (&mut r))`, `(rand.shuffle (&mut r) (&mut v))`, `(rand.entropy)` |
| endian | `(bytes.rd_le (& b) at n)` and `(bytes.rd_be (& b) at n)` for `n` in 1..8. `(bytes.wr_le)` and `(bytes.wr_be)` append |
| varint | `(bytes.uvar)`, `(bytes.svar)`, `(bytes.rd_uvar)`, `(bytes.rd_svar)`. A short buffer is `bytes.short` |
| crc32 | `(bytes.crc32 (& b))` IEEE |
| percent-encoding | `(url.encode (& s))` and `(url.decode (& s))`. Unreserved bytes stay. Space is `%20`. Decode requires valid UTF-8 |
| url split | `(url.parse (& s))` returns `(opt url)` with `scheme`, `host`, `port`, `path`, and `query`. A missing port is `0` |
| sha256 | `(sha256.sha256 (& msg))` is 32 raw bytes. `(sha.b64_decode (& s))` returns `(opt str)` |
| json number | kind `2` stores the number text, including floats |
| json text | `(json.show (& v))` and `(json.pretty (& v) indent)`. `(json.quote)` is unchanged |

## Threads

| task | form |
|------|------|
| start pool | `(thread.start n)` returns `(res pool str)`. `n < 1` is `thread.start\|n=` |
| submit | `(thread.submit (&mut p) work)`. `work` is `(clos () unit)` and can capture a binding. The pool grows when the queue is longer than the worker count |
| stop | `(thread.stop (&mut p))` joins. Drop does this too |
| spawn | `(thread.spawn work)` returns `(handle T)`. `work` is `(clos () T)` |
| join | `(thread.join h)` waits and returns `T`. Drop joins and drops the value |
| mutex | `(mutex.new v)` then `(mutex.lock (&mut m) body)`. `body` is `(clos ((&mut T)) unit)`, so the guard cannot be stored |
| channel | `(chan.new)`, `(chan.send (& c) v)` moves `v` and returns `bool`, `(chan.recv (& c))` and `(chan.try_recv (& c))` return `(opt T)`, `(chan.close (& c))`. `(chan.clone (& c))` shares it. There is no `Send` trait |

## HTTP

| task | form |
|------|------|
| parse request | `(http.parse_req (& text))` returns `(res req str)` |
| parse response | `(http.parse_resp (& text))` returns `(res resp str)` |
| render | `(http.show_req (& r))` and `(http.show_resp (& r))` |
| respond | `(http.respond code reason ty body)` sets `Content-Type` and `Content-Length` |
| header | `(http.header (& headers) name)` returns `(opt str)`, name compared in lower case |
| serve | `(http.serve fd root (& routes) stop_after)` returns `(res i64 str)`. `0` runs until killed. A route is method, path, and `(fn ((& req) (ptr u8)) resp)`. A path ending in `*` is a prefix. `{name}` captures one segment |
| chunked body | `(http_body.write_chunked body trailers)` builds a chunked body. `parse_req` decodes chunked requests and trailers |
| websocket | `Upgrade: websocket` and `Sec-WebSocket-Version: 13` answers `101`. Text, binary, ping, and close are framed messages |
| graphql | `POST /graphql` body is the query text, or JSON when `Content-Type` is `application/json`. `(gql.exec schema q vars)` returns JSON |
| soap | `POST /soap` body is one SOAP 1.1 envelope. `(soap.exec (& q))` returns XML |
| grpc | `(grpc.serve fd)` answers unary protobuf calls on many HTTP/2 streams |
| mqtt | `(mqtt.serve fd user pass stop_after)` is an MQTT 3.1.1 broker |

Program output destined for a model is one record per line, `|` separated, as in
[../spec/errors.md](../spec/errors.md). Diagnostics go to stderr, data to stdout.
