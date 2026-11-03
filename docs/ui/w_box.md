# ui.w_box

Container, padding, sized box, align, and center.

Source: `ui/w_box.llr`. Import with `(use ui.w_box)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `w_box.container` | `((c blend.color) (l f32) (t f32) (r f32) (b f32) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_box.padding` | `((l f32) (t f32) (r f32) (b f32) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_box.sized` | `((w f32) (h f32) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_box.align` | `((x f32) (y f32) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_box.center` | `((child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |

## Tests

`w_box_ty`
