# ui.host_android

Android host. The shim object is chosen at link time.

Source: `ui/host_android.llr`. Import with `(use ui.host_android)` for module functions; types and methods need no import.

## Types

- `(struct host_android (s host_shim.slot))`

## Functions

| name | params | returns |
|------|--------|---------|
| `host_android.open` | `((w i64) (h i64))` | `host_android` |
| `host_android.drop` | `((self (&mut host_android)))` | `unit` |

## Tests

`host_android_kind`
