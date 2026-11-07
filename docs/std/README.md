# std index

| module | role |
|--------|------|
| [sys](sys.md) | Panic runtime. `sys.os_id` is `1` on POSIX, `2` on Windows, and `3` on wasm32. Not for programs; std only. |
| [sys_posix](sys_posix.md) | POSIX externs merged into `std.sys`. |
| [sys_windows](sys_windows.md) | Windows externs merged into `std.sys`. |
| [sys_wasm](sys_wasm.md) | WASI preview1 externs merged into `std.sys` when the target contains `wasm32`. |
| [alloc](alloc.md) | Free-list `malloc`, `realloc`, and `free` over a grow hook. |
| [str](str.md) | Owned byte string `str`. Literals are `(& str)`. |
| [str_more](str_more.md) | More `str` methods: case, trim sides, UTF-8, radix, and fixed floats. |
| [vec](vec.md) | Growable array `(vec T)`. |
| [vec_more](vec_more.md) | More `vec` methods: clone, sort_by, retain, each, map, and filter. |
| [opt](opt.md) | Optional `(opt T)`: `(opt.some x)` / `(opt.none)`. |
| [res](res.md) | Result `(res T E)`: `(res.ok x)` / `(res.err e)`. |
| [io](io.md) | Standard streams. |
| [os](os.md) | Process environment. |
| [fs](fs.md) | Files and directories. Errors are `res.err` strings like `io.read|path=x`. |
| [net](net.md) | TCP and UDP. `connect` takes a name or an IP literal. |
| [net_dns](net_dns.md) | DNS lookup and Linux sockaddr literals used by `net`. |
| [thread](thread.md) | A pthread pool, plus `spawn` and `join`. |
| [sync](sync.md) | `mutex` and `chan`. Sending moves the value. No `Send` trait. |
| [http](http.md) | HTTP/1.1, chunked bodies, route templates, and WebSocket messages. |
| [http_body](http_body.md) | Chunked bodies and `Content-Length`. |
| [http_mime](http_mime.md) | Test only. No public functions. |
| [http_route](http_route.md) | Exact paths, a trailing `*`, and `{name}` segments. |
| [http_type](http_type.md) | Content-Type from a file suffix. Unknown suffixes stay text/plain. |
| [http_ws](http_ws.md) | WebSocket frames and continuation messages. |
| [sha](sha.md) | SHA-1 digest and base64, used by the WebSocket handshake. |
| [sha256](sha256.md) | SHA-256 digest, 32 raw bytes. |
| [gql](gql.md) | GraphQL execution against a schema. |
| [gql_parse](gql_parse.md) | GraphQL document parser. |
| [gql_schema](gql_schema.md) | GraphQL schema parser and the builtin schema. |
| [xml](xml.md) | XML tree with attributes, namespaces, comments, and CDATA. |
| [soap](soap.md) | SOAP 1.1 envelope, header, and Fault. |
| [grpc](grpc.md) | Unary gRPC. HTTP/2, HPACK, and protobuf messages. |
| [h2](h2.md) | HTTP/2 frames. |
| [hpack](hpack.md) | HPACK static table, dynamic table, and Huffman strings. |
| [hpack_huff](hpack_huff.md) | HPACK Huffman bit strings. |
| [pb](pb.md) | Protobuf wire types. |
| [cmp](cmp.md) | `Eq`, `Ord`, `Hash`, and `Clone`. |
| [mqtt](mqtt.md) | MQTT 3.1.1 broker. QoS 0-2, wildcards, and auth. |
| [mqtt_pkt](mqtt_pkt.md) | MQTT remaining length and strings. |
| [mqtt_sess](mqtt_sess.md) | MQTT topic filters. |
| [path](path.md) | Path strings. `clean` and `join` emit `/`. Windows also accepts `\`. |
| [map](map.md) | Hash map `(map K V)`. Keys need `Hash` and `Eq`. |
| [set](set.md) | Set `(set K)` over `(map K unit)`. |
| [deque](deque.md) | Ring buffer `(deque T)`. |
| [math](math.md) | `f64` and `f32` abs, min, max, floor, ceil, round, sqrt, integers, clamp, lerp, and fmod. No libm. |
| [math_trig](math_trig.md) | sin, cos, tan, asin, acos, atan2, exp, ln, log2, log10, and pow. |
| [fmt](fmt.md) | Pipe records, hex, fixed floats, and padding. |
| [json](json.md) | Minimal JSON encoding, `show`, and `pretty`. |
| [json_read](json_read.md) | JSON objects, arrays, strings, numbers, bools, and null. |
| [cli](cli.md) | Argument parsing for tools. |
| [sexp](sexp.md) | S-expression reader used by `llr-fmt` and `llr-doc`. |
| [scan](scan.md) | Cursor over `(& str)`: peek, names, and hex digits. |
| [ast](ast.md) | Surface grammar parsed from a sexp tree. `llrc check` is the checker; `llr-check` runs it. |
| [ast_expr](ast_expr.md) | Expression and type forms parsed from a sexp tree. |
| [glob](glob.md) | Glob matching and directory walking. |
| [re](re.md) | Backtracking regex: `. ^ $ * + ? {n,m} [..] [^..] \d \w \s \b \N ( ) |`. |
| [diff](diff.md) | Line diff. Shared ends are exact. The middle is a minimal LCS. |
| [time](time.md) | Wall clock, monotonic clock, sleep, and UTC dates. |
| [rand](rand.md) | xorshift64* `(rng)` and `entropy`. |
| [bytes](bytes.md) | Little-endian and big-endian integers, LEB128, and CRC-32. |
| [url](url.md) | Percent-encoding and a small URL split. |
| [test](test.md) | Test harness used by `llrc build --test` and `llr-test`. |

Rules: one module per file, every function has a test form in its file, `llr-doc` regenerates the signature tables (see docs/workflows/std.md).
