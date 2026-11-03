# ui.m_toggle

Checkbox, switch, and a slider that owns a value from 0 to 100.

Source: `ui/m_toggle.llr`. Import with `(use ui.m_toggle)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `m_toggle.checkbox` | `((on bool) (msg i64))` | `(box (dyn widget.widget))` |
| `m_toggle.switch` | `((on bool) (msg i64))` | `(box (dyn widget.widget))` |
| `m_toggle.slider` | `((v i64))` | `(box (dyn widget.widget))` |

## Tests

`m_toggle_name`
