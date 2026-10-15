# std.json

Minimal JSON encoding.

Source: `std/json.llr`. Import with `(use std.json)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `json.quote` | `((s (& str)))` | `str` |
| `json.obj` | `((keys (& (vec str))) (vals (& (vec str))))` | `str` |
| `json.arr` | `((vals (& (vec str))))` | `str` |

## Tests

`json_quote`
