# std.gql

A fixed GraphQL subset. The query text is the input. The result is JSON. There is no schema, no variables, no fragments, no aliases, and no nested selection set.

```
query := '{' field* '}'
field := name | name '(' arg (',' arg)* ')'
arg   := name ':' integer
```

Whitespace is skipped. `hello` is `"world"`. `ping` is `"pong"`. `add` sums integers `a` and `b`; a missing argument is 0. A good query is `{"data":{...}}`. A bad query is `{"errors":[{"message":"gql.parse"}]}`. An unknown field is `{"errors":[{"message":"gql.field|name="}]}`.

Source: `std/gql.llr`. Import with `(use std.gql)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `gql.exec` | `((q (& str)))` | `str` |

## Tests

`gql_fields`, `gql_http`
