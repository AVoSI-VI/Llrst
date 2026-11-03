# ui.m_button

Filled button and icon button. The icon is a label in the bundled font.

Source: `ui/m_button.llr`. Import with `(use ui.m_button)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `m_button.button` | `((label str) (msg i64))` | `(box (dyn widget.widget))` |
| `m_button.icon_button` | `((label str) (msg i64))` | `(box (dyn widget.widget))` |

## Tests

`m_button_name`
