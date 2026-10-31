# ui.event

Pointer, key, text, and script records. Pointer kind: 0 down, 1 move, 2 up, 3 cancel, 4 scroll.

Source: `ui/event.llr`. Import with `(use ui.event)` for module functions; types and methods need no import.

## Types

- `(enum event (pub (pointer i64 f32 f32 f32 f32)) (pub (key i64 i64 i64)) (pub (text str)) (pub (frame)) (pub (quit)))`

## Constants

| name | type | value |
|------|------|-------|
| `event.down` | `i64` | `0` |
| `event.move` | `i64` | `1` |
| `event.up` | `i64` | `2` |
| `event.cancel` | `i64` | `3` |
| `event.scroll` | `i64` | `4` |
| `event.tab` | `i64` | `9` |

## Functions

| name | params | returns |
|------|--------|---------|
| `event.parse_line` | `((s (& str)))` | `(opt event)` |

## Tests

`event_parse`
