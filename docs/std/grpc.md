# std.grpc

One unary gRPC call on a small HTTP/2 subset. The message is plain text inside one length-prefixed frame. There is no protobuf compiler.

The client starts with the 24-byte preface `PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n`, then one empty SETTINGS frame (type 4, stream 0). The server writes an empty SETTINGS and a SETTINGS ACK.

A frame is 9 bytes plus a payload: 24-bit length, type, flags, and a 31-bit stream id. A payload longer than 4096 bytes is `h2.frame`. A bad preface is `h2.preface`.

HEADERS (type 1) is stream 1 with END_HEADERS and no other flags. Each header is a name length, the name, a value length, and the value. Both lengths are one byte under 127. The bytes are raw. There is no Huffman coding and no dynamic table. `:method` must be `POST`. `:path` is required. Anything else in the block is `h2.headers`. WINDOW_UPDATE and PING are skipped.

The DATA payload is one gRPC frame: a `0` compression byte, a big-endian length, and that many text bytes. A short buffer or a non-zero compression flag is `grpc.frame`. `/hello` is `world`. `/ping` is `pong`. `/add` reads two integers separated by one space; a missing integer is 0. A bad integer is `grpc.arg`. An unknown path is `grpc.method|name=`.

The answer is HEADERS (`:status` `200`, `content-type` `application/grpc`, `grpc-status` `0` or `12`) and one END_STREAM DATA frame. Status `12` carries `grpc.method` or `grpc.arg`. There are no trailers. Only stream 1 carries the call.

Source: `std/grpc.llr`. Import with `(use std.grpc)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `grpc.pack` | `((msg (& str)))` | `str` |
| `grpc.exec` | `((path (& str)) (msg (& str)))` | `(res str str)` |
| `grpc.serve` | `((fd i64))` | `(res i64 str)` |

`grpc.serve` accepts one connection, accumulates bytes until the call is complete, sends the reply, and returns `1`.

## Tests

`grpc_fields`, `grpc_http`
