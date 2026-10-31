# std.gql_parse

GraphQL document. One operation, nested selections, aliases, fragments, and variables.

Source: `std/gql_parse.llr`. Import with `(use std.gql_parse)` for module functions; types and methods need no import.

## Types

- `(struct doc (nodes (vec node)) (kids (vec i64)) (pub (root i64)) (pub (frags (vec i64))) (pub (vars (vec i64))))`

## Functions

| name | params | returns |
|------|--------|---------|
| `gql_parse.read` | `((src (& str)))` | `(res doc str)` |
| `gql_parse.input` | `((src (& str)))` | `(res doc str)` |
| `gql_parse.kind` | `((d (& doc)) (n i64))` | `u8` |
| `gql_parse.text` | `((d (& doc)) (n i64))` | `(& str)` |
| `gql_parse.alt` | `((d (& doc)) (n i64))` | `(& str)` |
| `gql_parse.count` | `((d (& doc)) (n i64))` | `i64` |
| `gql_parse.child` | `((d (& doc)) (n i64) (k i64))` | `i64` |

## Tests

`gql_parse_hello`
