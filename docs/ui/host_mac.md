# ui.host_mac

Cocoa host. The shim object is chosen at link time.

Source: `ui/host_mac.llr`. Import with `(use ui.host_mac)` for module functions; types and methods need no import.

## Types

- `(struct host_mac (s host_shim.slot))`

## Functions

| name | params | returns |
|------|--------|---------|
| `host_mac.open` | `((w i64) (h i64))` | `host_mac` |
| `host_mac.drop` | `((self (&mut host_mac)))` | `unit` |
