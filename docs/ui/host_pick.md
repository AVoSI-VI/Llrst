# ui.host_pick

Picks the desktop host from the operating system.

Source: `ui/host_pick.llr`. Import with `(use ui.host_pick)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `host_pick.which` | `()` | `i64` |
| `host_pick.open` | `((w i64) (h i64))` | `(box (dyn host.host))` |

## Tests

`host_pick_which`
