# std.ast

Surface grammar parsed from a sexp tree. llrc remains the reference checker.

Source: `std/ast.llr`. Import with `(use std.ast)` for module functions; types and methods need no import.

## Types

- `(struct ty (pub (kind u8)) (pub (text str)) (pub (args (vec ty))) (pub (line i64)) (pub (col i64)))`
- `(struct expr (pub (kind u8)) (pub (text str)) (pub (mut bool)) (pub (kids (vec expr))) (pub (tys (vec ty))) (pub (line i64)) (pub (col i64)))`
- `(struct param (pub (name str)) (pub (ty ty)))`
- `(struct field (pub (name str)) (pub (ty ty)) (pub (is_pub bool)))`
- `(struct variant (pub (name str)) (pub (tys (vec ty))))`
- `(struct item (pub (kind u8)) (pub (name str)) (pub (is_pub bool)) (pub (gens (vec str))) (pub (params (vec param))) (pub (fields (vec field))) (pub (variants (vec variant))) (pub (ret ty)) (pub (body (vec expr))) (pub (fns (vec item))) (pub (line i64)) (pub (col i64)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `ast.parse` | `((t (& tree)))` | `(res (vec item) str)` |
| `ast.parse_src` | `((src (& str)))` | `(res (vec item) str)` |

## Tests

`ast_fn`, `ast_struct`, `ast_enum_impl`, `ast_bad`
