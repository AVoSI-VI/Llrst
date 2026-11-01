# ui.w_anim

Builder and implicit leaves. A leaf keeps its child; the ticker rewrites the render desc.

Source: `ui/w_anim.llr`. Import with `(use ui.w_anim)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `w_anim.builder` | `((dur i64) (cur i64) (body (clos (f32) (box (dyn widget.widget)))))` | `(box (dyn widget.widget))` |
| `w_anim.padding` | `((l f32) (t f32) (r f32) (b f32) (dur i64) (cur i64) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_anim.opacity` | `((a f32) (dur i64) (cur i64) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_anim.container` | `((c blend.color) (l f32) (t f32) (r f32) (b f32) (dur i64) (cur i64) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |

## Tests

`w_anim_names`, `w_anim_msg`, `w_anim_frames`, `w_anim_pad`
