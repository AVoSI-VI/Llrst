# ui.m_layer

Dialog scrim and a snack bar that leaves after a short tick.

Source: `ui/m_layer.llr`. Import with `(use ui.m_layer)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `m_layer.dialog` | `((msg i64) (body (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `m_layer.snack_bar` | `((text str))` | `(box (dyn widget.widget))` |

## Tests

`m_layer_name`
