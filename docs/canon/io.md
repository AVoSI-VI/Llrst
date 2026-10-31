# Canon: io

| task | form |
|------|------|
| stdout line | `(io.puts (& s))` appends `\n` |
| stdout raw | `(io.put (& s))` |
| stderr line | `(io.eputs (& s))` |
| stderr raw | `(io.eput (& s))` |
| read all stdin | `(io.stdin)` returns `str` |
| read file | `(fs.read (& path))` returns `(res str str)` |
| write file | `(fs.write (& path) (& data))` returns `(res unit str)` |
| append file | `(fs.append (& path) (& data))` returns `(res unit str)` |
| exists | `(fs.exists (& path))` |
| is dir | `(fs.is_dir (& path))` |
| remove file | `(fs.remove (& path))` returns `(res unit str)` |
| make dir | `(fs.mkdir (& path))` returns `(res unit str)` |
| make dir tree | `(fs.mkdir_all (& path))` returns `(res unit str)`. `.` and an existing directory are success |
| remove tree | `(fs.remove_all (& path))` returns `(res unit str)`. A file is unlinked; a directory is removed with its contents |
| rename | `(fs.rename (& from) (& to))` returns `(res unit str)` |
| list dir | `(fs.list (& path))` returns `(res (vec str) str)`, names only, sorted |
| mtime | `(fs.mtime (& path))` returns `(res i64 str)` seconds |
| size | `(fs.size (& path))` returns `(res i64 str)` bytes |
| join path | `(path.join (& a) (& b))` |
| base name | `(path.base (& p))` |
| stem | `(path.stem (& p))` drops the extension |
| dir name | `(path.dir (& p))` |
| extension | `(path.ext (& p))` without dot |
| absolute | `(path.is_abs (& p))` |
| clean | `(path.clean (& p))` drops `.` and duplicate slashes, keeps `..` |
| args | `(os.args)` returns `(vec str)` including argv0 |
| env | `(os.env (& name))` returns `(opt str)` |
| shell | `(os.shell (& cmd))` returns the exit code |
| capture | `(os.capture (& cmd))` returns `cmd` with stdout and the exit code |
| arch | `(os.arch)` returns `(& str)` |
| tty | `(os.is_tty fd)` |
| exit | `(os.exit code)` |
| cwd | `(os.cwd)` returns `(res str str)`. Failure, including a path past 1 MiB, is `io.cwd` |
| listen | `(net.listen port)` returns `(res i64 str)`, the listening fd. Port 0 is any port. Backlog 1 |
| local port | `(net.local_port fd)` returns `(res i64 str)` |
| accept | `(net.accept fd)` blocks and returns `(res i64 str)` |
| connect | `(net.connect host port)` returns `(res i64 str)`. `host` is a dotted IPv4 |
| recv | `(net.recv fd n)` returns `(res str str)`. An empty string is end of stream |
| send | `(net.send fd (& data))` returns `(res unit str)` and writes every byte |
| close socket | `(net.close fd)` |
| poll serve | `(net.poll_serve fd max each)` returns `(res i64 str)`, the number of reads. `each` is `(clos (i64 (& str)) bool)`. `max < 0` does not cap clients |

## Threads

| task | form |
|------|------|
| start pool | `(thread.start n)` returns `(res pool str)`. `n < 1` is `thread.start\|n=` |
| submit | `(thread.submit (&mut p) work)`. `work` is `(clos () unit)` and can capture a binding. The pool grows when the queue is longer than the worker count |
| stop | `(thread.stop (&mut p))` joins. Drop does this too |

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
