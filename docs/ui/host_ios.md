# ui.host_ios

iOS host. The shim object is chosen at link time.

Source: `ui/host_ios.llr`. Import with `(use ui.host_ios)` for module functions; types and methods need no import.

## Types

- `(struct host_ios (s host_shim.slot))`

## Functions

| name | params | returns |
|------|--------|---------|
| `host_ios.open` | `((w i64) (h i64))` | `host_ios` |
| `host_ios.drop` | `((self (&mut host_ios)))` | `unit` |

## Tests

`host_ios_kind`
