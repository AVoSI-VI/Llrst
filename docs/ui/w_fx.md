# ui.w_fx

Opacity, clip, and transform. They are repaint boundaries and do not change layout.

Source: `ui/w_fx.llr`. Import with `(use ui.w_fx)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `w_fx.opacity` | `((a f32) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_fx.clip_rect` | `((child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_fx.transform` | `((tx f32) (ty f32) (sx f32) (sy f32) (rot f32) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |

## Tests

`w_fx_ty`
