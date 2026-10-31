# ui.render_paint

Paint the render tree into one bitmap. Opacity, clip, and transform isolate their children.

Source: `ui/render_paint.llr`. Import with `(use ui.render_paint)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `render_paint.paint` | `((rt (& render.rtree)) (g (&mut render_box.fonts)) (id u32) (bm (&mut bitmap.bitmap)) (vw f32) (vh f32) (sc f32))` | `unit` |
