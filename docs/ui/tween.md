# ui.tween

Linear interpolation. Color bytes are the stored premultiplied channels.

Source: `ui/tween.llr`. Import with `(use ui.tween)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `tween.num` | `((a f32) (b f32) (t f32))` | `f32` |
| `tween.color` | `((a (& blend.color)) (b (& blend.color)) (t f32))` | `blend.color` |
| `tween.rect` | `((a (& geom.rect)) (b (& geom.rect)) (t f32))` | `geom.rect` |
| `tween.size` | `((a (& geom.size)) (b (& geom.size)) (t f32))` | `geom.size` |

## Tests

`tween_lerp`
