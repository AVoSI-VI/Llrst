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
| [net](net.md) | IPv4 on 127.0.0.1. `poll_serve` holds many connections on one thread. |
| [thread](thread.md) | Fixed pthread pool. A job is a function pointer plus a context pointer. |
| [http](http.md) | HTTP/1.1 request and response in one buffer. No chunked bodies. |
| [sha](sha.md) | SHA-1 digest and base64, used by the WebSocket handshake. |
| [gql](gql.md) | Fixed GraphQL subset. Query text in, JSON out. |
| [xml](xml.md) | One XML document of elements and text. |
| [soap](soap.md) | One SOAP envelope in, one XML envelope out. |
| [grpc](grpc.md) | One unary gRPC call. Length-prefixed text, no protobuf. |
| [cmp](cmp.md) | `Eq`, `Ord`, `Hash`, and `Clone`. |
| [mqtt](mqtt.md) | One MQTT connection. Connect, subscribe, and QoS 0 publish. |
| [path](path.md) | Path string helpers; `/` only. |
| [map](map.md) | Hash map `(map K V)`. Keys need `Hash` and `Eq`. |
| [fmt](fmt.md) | Pipe-record formatting for model-facing output. |
| [json](json.md) | Minimal JSON encoding. |
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
