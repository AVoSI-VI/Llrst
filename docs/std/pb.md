# std.pb

Protobuf wire types 0, 1, 2, and 5. Unknown fields are skipped.

Source: `std/pb.llr`. Import with `(use std.pb)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `pb.put_int` | `((out (&mut str)) (field i64) (n i64))` | `unit` |
| `pb.put_str` | `((out (&mut str)) (field i64) (s (& str)))` | `unit` |
| `pb.get_int` | `((buf (& str)) (field i64))` | `(res i64 str)` |
| `pb.get_str` | `((buf (& str)) (field i64))` | `(res str str)` |

## Tests

`pb_round`
