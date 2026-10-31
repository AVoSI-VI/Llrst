# std index

| module | role |
|--------|------|
| [sys](sys.md) | Panic runtime. `sys.os_id` is `1` on POSIX and `2` on Windows. Not for programs; std only. |
| [sys_posix](sys_posix.md) | POSIX externs merged into `std.sys`. |
| [sys_windows](sys_windows.md) | Windows externs merged into `std.sys`. |
| [str](str.md) | Owned byte string `str`. Literals are `(& str)`. |
| [vec](vec.md) | Growable array `(vec T)`. |
| [opt](opt.md) | Optional `(opt T)`: `(opt.some x)` / `(opt.none)`. |
| [res](res.md) | Result `(res T E)`: `(res.ok x)` / `(res.err e)`. |
| [io](io.md) | Standard streams. |
| [os](os.md) | Process environment. |
| [fs](fs.md) | Files and directories. Errors are `res.err` strings like `io.read|path=x`. |
| [net](net.md) | IPv4. `connect` takes a dotted quad. `poll_serve` grows with the clients. |
| [thread](thread.md) | A pthread pool that grows. A job is a closure. |
| [http](http.md) | HTTP/1.1, chunked bodies, route templates, and WebSocket messages. |
| [http_body](http_body.md) | Chunked bodies and `Content-Length`. |
| [http_route](http_route.md) | Exact paths, a trailing `*`, and `{name}` segments. |
| [http_ws](http_ws.md) | WebSocket frames and continuation messages. |
| [sha](sha.md) | SHA-1 digest and base64, used by the WebSocket handshake. |
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
| [path](path.md) | Path string helpers; `/` only. |
| [map](map.md) | Hash map `(map K V)`. Keys need `Hash` and `Eq`. |
| [fmt](fmt.md) | Pipe-record formatting for model-facing output. |
| [json](json.md) | Minimal JSON encoding. |
| [json_read](json_read.md) | JSON objects, arrays, strings, ints, bools, and null. |
| [cli](cli.md) | Argument parsing for tools. |
| [sexp](sexp.md) | S-expression reader used by `llr-fmt` and `llr-doc`. |
| [ast](ast.md) | Surface grammar parsed from a sexp tree. `llrc` remains the checker. |
| [ast_expr](ast_expr.md) | Expression and type forms parsed from a sexp tree. |
| [check](check.md) | Typed tree. Borrow checking stays in `llrc`. |
| [check_more](check_more.md) | Impl shape and generic bounds for `llr-check`. |
| [check_ty](check_ty.md) | Type display, generic arity, and `(dyn Trait)` object safety. |
| [check_expr](check_expr.md) | Expression types, resolved callees, and builtin ids. |
| [glob](glob.md) | Glob matching and directory walking. |
| [re](re.md) | Backtracking regex: `. ^ $ * + ? {n,m} [..] [^..] \d \w \s \b \N ( ) |`. |
| [diff](diff.md) | Line diff. Shared ends are exact. The middle is a minimal LCS. |
| [test](test.md) | Test harness used by `llrc build --test` and `llr-test`. |

Rules: one module per file, every function has a test form in its file, `llr-doc` regenerates the signature tables (see docs/workflows/std.md).
