# ui.host_x11

X11 host. The shim object is chosen at link time.

Source: `ui/host_x11.llr`. Import with `(use ui.host_x11)` for module functions; types and methods need no import.

## Types

- `(struct host_x11 (s host_shim.slot))`

## Functions

| name | params | returns |
|------|--------|---------|
| `host_x11.open` | `((w i64) (h i64))` | `host_x11` |
| `host_x11.drop` | `((self (&mut host_x11)))` | `unit` |
