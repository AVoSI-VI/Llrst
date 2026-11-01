# ui.event

Pointer, key, text, and script records. `take_rec` reads the 64-byte host record.

Source: `ui/event.llr`. Import with `(use ui.event)` for module functions; types and methods need no import.

## Types

- `(enum event (pub (pointer i64 f32 f32 f32 f32)) (pub (key i64 i64 i64)) (pub (text str)) (pub (frame)) (pub (quit)) (pub (life i64)) (pub (inset f32 f32 f32 f32)))`

## Constants

| name | type | value |
|------|------|-------|
| `event.down` | `i64` | `0` |
| `event.move` | `i64` | `1` |
| `event.up` | `i64` | `2` |
| `event.cancel` | `i64` | `3` |
| `event.scroll` | `i64` | `4` |
| `event.tab` | `i64` | `9` |
| `event.shift` | `i64` | `1` |
| `event.ctrl` | `i64` | `2` |
| `event.alt` | `i64` | `4` |
| `event.meta` | `i64` | `8` |
| `event.rec_bytes` | `i64` | `64` |
| `event.rec_pointer` | `i64` | `1` |
| `event.rec_key` | `i64` | `2` |
| `event.rec_text` | `i64` | `3` |
| `event.rec_frame` | `i64` | `4` |
| `event.rec_quit` | `i64` | `5` |
| `event.rec_life` | `i64` | `6` |
| `event.rec_inset` | `i64` | `7` |
| `event.pause` | `i64` | `0` |
| `event.resume` | `i64` | `1` |
| `event.destroy` | `i64` | `2` |

## Functions

| name | params | returns |
|------|--------|---------|
| `event.parse_line` | `((s (& str)))` | `(opt event)` |
| `event.take_rec` | `((buf (ptr u8)) (n i64) (out (&mut (vec event))))` | `unit` |

## Tests

`event_parse`, `event_rec_keys`, `event_rec_text`, `event_rec_life`
