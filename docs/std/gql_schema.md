# std.gql_schema

GraphQL schema. type, fields, arguments, and Int, String, Boolean, or an object type.

Source: `std/gql_schema.llr`. Import with `(use std.gql_schema)` for module functions; types and methods need no import.

## Types

- `(struct arg (pub (name str)) (pub (ty str)))`
- `(struct field (pub (name str)) (pub (ty str)) (pub (args (vec arg))))`
- `(struct ty (pub (name str)) (pub (fields (vec field))))`
- `(struct schema (pub (query str)) (pub (types (vec ty))))`

## Functions

| name | params | returns |
|------|--------|---------|
| `gql_schema.read` | `((src (& str)))` | `(res schema str)` |
| `gql_schema.builtin` | `()` | `schema` |
| `gql_schema.find_ty` | `((s (& schema)) (name (& str)))` | `(opt i64)` |
| `gql_schema.find_field` | `((t (& ty)) (name (& str)))` | `(opt i64)` |

## Tests

`gql_schema_builtin`
