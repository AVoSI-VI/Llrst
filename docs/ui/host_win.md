# ui.host_win

Win32 host. The shim object is chosen at link time.

Source: `ui/host_win.llr`. Import with `(use ui.host_win)` for module functions; types and methods need no import.

## Types

- `(struct host_win (s host_shim.slot))`

## Functions

| name | params | returns |
|------|--------|---------|
| `host_win.open` | `((w i64) (h i64))` | `host_win` |
| `host_win.drop` | `((self (&mut host_win)))` | `unit` |
