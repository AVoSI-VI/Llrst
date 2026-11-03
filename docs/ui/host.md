# ui.host

Platform surface. The framework paints; a host only presents pixels and reports events.

Source: `ui/host.llr`. Import with `(use ui.host)` for module functions; types and methods need no import.

## Traits

| name | params | returns |
|------|--------|---------|
| `host.size` | `((& Self))` | `geom.size` |
| `host.scale` | `((& Self))` | `f32` |
| `host.present` | `((&mut Self) (& bitmap.bitmap))` | `unit` |
| `host.poll` | `((&mut Self) (&mut (vec event.event)))` | `unit` |
| `host.request_frame` | `((&mut Self))` | `unit` |
| `host.now_ms` | `((& Self))` | `i64` |
| `host.set_cursor` | `((&mut Self) i64)` | `unit` |
| `host.clipboard_get` | `((& Self))` | `str` |
| `host.clipboard_set` | `((&mut Self) (& str))` | `unit` |
| `host.text_input` | `((&mut Self) bool (& geom.rect))` | `unit` |
| `host.quit` | `((&mut Self))` | `unit` |
| `host.history` | `((&mut Self) i64)` | `unit` |

## Constants

| name | type | value |
|------|------|-------|
| `host.cursor_arrow` | `i64` | `0` |
| `host.cursor_text` | `i64` | `1` |
| `host.cursor_hand` | `i64` | `2` |
| `host.cursor_cross` | `i64` | `3` |
| `host.cursor_wait` | `i64` | `4` |

## Functions

| name | params | returns |
|------|--------|---------|
