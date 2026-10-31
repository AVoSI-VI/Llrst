# std.gql

GraphQL execution against a schema. One operation, nested selections, aliases, fragments, and variables. `(gql.exec schema query vars)` runs `query`. `vars` is a GraphQL input object or empty. The builtin schema answers `hello`, `ping`, `add`, and `pair`. A missing variable is `gql.var|name=`. Subscriptions, directives, and introspection are `gql.parse`. `POST /graphql` with `Content-Type: application/json` reads `{"query","variables"}`. Any other body is the query text.

Source: `std/gql.llr`. Import with `(use std.gql)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `gql.exec` | `((sc (& gql_schema.schema)) (q (& str)) (vars (& str)))` | `str` |
| `gql.answer` | `((hs (& (vec str))) (body (& str)))` | `str` |

## Tests

`gql_fields`, `gql_answer`, `gql_http`
