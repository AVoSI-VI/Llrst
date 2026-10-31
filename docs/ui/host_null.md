# ui.host_null

Headless host. Size and events come from argv; present writes a PPM when --shot is set.

Source: `ui/host_null.llr`. Import with `(use ui.host_null)` for module functions; types and methods need no import.

## Types

- `(struct host_null (w i64) (h i64) (sc f32) (shot str) (lines (vec str)) (at i64) (cursor i64) (clip str) (text_on bool) (text_r geom.rect) (quitb bool) (clock i64) (step i64) (frames i64) (want bool))`

## Functions

| name | params | returns |
|------|--------|---------|
| `host_null.make` | `((w i64) (h i64))` | `host_null` |
| `host_null.load_text` | `((self (&mut host_null)) (s (& str)))` | `unit` |
| `host_null.load_file` | `((self (&mut host_null)) (path (& str)))` | `unit` |
| `host_null.take_args` | `((self (&mut host_null)))` | `unit` |
| `host_null.from_args` | `()` | `host_null` |
| `host_null.frames` | `((self (& host_null)))` | `i64` |
| `host_null.quitting` | `((self (& host_null)))` | `bool` |
| `host_null.cursor_id` | `((self (& host_null)))` | `i64` |
| `host_null.clip_text` | `((self (& host_null)))` | `str` |
| `host_null.text_on` | `((self (& host_null)))` | `bool` |

## Tests

`host_null_poll`
