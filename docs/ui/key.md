# ui.key

Optional widget key. Both absent keys match; a present key matches only the same id.

Source: `ui/key.llr`. Import with `(use ui.key)` for module functions; types and methods need no import.

## Types

- `(struct key (pub (on bool)) (pub (id u32)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `key.none` | `()` | `key` |
| `key.value` | `((id u32))` | `key` |
| `key.same` | `((a (& key)) (b (& key)))` | `bool` |

## Tests

`key_same`
