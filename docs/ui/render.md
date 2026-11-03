# ui.render

Render arena. Ids stay put; a dead node is skipped. Custom objects are leaves.

Source: `ui/render.llr`. Import with `(use ui.render)` for module functions; types and methods need no import.

## Types

- `(struct rnode (pub (elem u32)) (pub (parent u32)) (pub (has_parent bool)) (pub (kids (vec u32))) (pub (ty i64)) (pub (x f32)) (pub (y f32)) (pub (w f32)) (pub (h f32)) (pub (rx f32)) (pub (ry f32)) (pub (cminw f32)) (pub (cmaxw f32)) (pub (cminh f32)) (pub (cmaxh f32)) (pub (has_c bool)) (pub (dirty_lay bool)) (pub (dirty_paint bool)) (pub (relayout_b bool)) (pub (repaint_b bool)) (pub (flex i64)) (pub (gesture bool)) (pub (has_pos bool)) (pub (px f32)) (pub (py f32)) (pub (pw f32)) (pub (ph f32)) (pub (live bool)) (pub (d widget.rdesc)))`
- `(struct rtree (pub (nodes (vec rnode))))`
- `(struct empty_r)`

## Traits

| name | params | returns |
|------|--------|---------|
| `render.layout` | `((&mut Self) (& render_box.constraints))` | `geom.size` |
| `render.paint` | `((& Self) (&mut canvas.canvas) f32 f32)` | `unit` |
| `render.hit` | `((& Self) f32 f32)` | `bool` |

## Functions

| name | params | returns |
|------|--------|---------|
| `render.empty_box` | `()` | `(box (dyn render))` |
| `render.new` | `()` | `rtree` |
| `render.live` | `((rt (& rtree)) (id u32))` | `bool` |
| `render.has_parent` | `((rt (& rtree)) (id u32))` | `bool` |
| `render.parent` | `((rt (& rtree)) (id u32))` | `u32` |
| `render.relayout_b` | `((rt (& rtree)) (id u32))` | `bool` |
| `render.repaint_b` | `((rt (& rtree)) (id u32))` | `bool` |
| `render.kids_of` | `((rt (& rtree)) (id u32))` | `(vec u32)` |
| `render.add` | `((rt (&mut rtree)) (elem u32) (parent u32) (has bool) (ty i64) (d widget.rdesc) (pd (& widget.pdata)))` | `u32` |
| `render.clear_kids` | `((rt (&mut rtree)) (id u32))` | `unit` |
| `render.add_kid` | `((rt (&mut rtree)) (parent u32) (id u32))` | `unit` |
| `render.touch` | `((rt (&mut rtree)) (id u32))` | `unit` |
| `render.set_rel` | `((rt (&mut rtree)) (id u32) (rx f32) (ry f32))` | `unit` |
| `render.place_abs` | `((rt (&mut rtree)) (id u32) (x f32) (y f32))` | `unit` |
| `render.replace_desc` | `((rt (&mut rtree)) (id u32) (ty i64) (d widget.rdesc) (gesture bool) (relayout bool) (repaint bool))` | `unit` |
| `render.set_pdata` | `((rt (&mut rtree)) (id u32) (pd (& widget.pdata)))` | `unit` |
| `render.kill` | `((rt (&mut rtree)) (id u32))` | `unit` |
| `render.visual_root` | `((rt (& rtree)))` | `(opt u32)` |

## Tests

`render_touch`
