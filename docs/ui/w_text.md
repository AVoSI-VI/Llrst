# ui.w_text

A single-style paragraph. Width wraps at the incoming max constraint.

Source: `ui/w_text.llr`. Import with `(use ui.w_text)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `w_text.text` | `((s (& str)) (size f32) (c blend.color))` | `(box (dyn widget.widget))` |

## Tests

`w_text_ty`
