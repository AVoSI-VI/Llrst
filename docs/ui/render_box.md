# ui.render_box

Box layout. A clean relayout boundary keeps its size when constraints match.

Source: `ui/render_box.llr`. Import with `(use ui.render_box)` for module functions; types and methods need no import.

## Types

- `(struct fonts (f (opt font.font)))`
- `(struct constraints (pub (min_w f32)) (pub (max_w f32)) (pub (min_h f32)) (pub (max_h f32)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `render_box.fonts_new` | `()` | `fonts` |
| `render_box.loose` | `((w f32) (h f32))` | `constraints` |
| `render_box.glyphs` | `((g (&mut fonts)) (text (& str)) (size f32) (max_w f32) (col blend.color))` | `(vec paint.stamp)` |
| `render_box.layout_tree` | `((rt (&mut render.rtree)) (g (&mut fonts)) (id u32) (w f32) (h f32))` | `geom.size` |
