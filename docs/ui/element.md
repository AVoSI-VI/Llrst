# ui.element

Element arena. Reconciliation keeps a child when its type id and key match.

Source: `ui/element.llr`. Import with `(use ui.element)` for module functions; types and methods need no import.

## Types

- `(struct elem (pub (ty i64)) (pub (k key.key)) (pub (role i64)) (pub (parent u32)) (pub (has_parent bool)) (pub (kids (vec u32))) (pub (render u32)) (pub (has_render bool)) (pub (widget (box (dyn widget.widget)))) (pub (st (box (dyn state.state)))) (pub (has_st bool)) (pub (dirty bool)) (pub (live bool)))`
- `(struct elems (pub (nodes (vec elem))) (pub (vw f32)) (pub (vh f32)) (pub (sc f32)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `element.new` | `()` | `elems` |
| `element.set_view` | `((es (&mut elems)) (w f32) (h f32) (sc f32))` | `unit` |
| `element.live_at` | `((es (& elems)) (id u32))` | `bool` |
| `element.ty_at` | `((es (& elems)) (id u32))` | `i64` |
| `element.role_at` | `((es (& elems)) (id u32))` | `i64` |
| `element.render_at` | `((es (& elems)) (id u32))` | `u32` |
| `element.has_render_at` | `((es (& elems)) (id u32))` | `bool` |
| `element.kids_at` | `((es (& elems)) (id u32))` | `(vec u32)` |
| `element.unmount` | `((es (&mut elems)) (rt (&mut render.rtree)) (id u32))` | `unit` |
| `element.update` | `((es (&mut elems)) (rt (&mut render.rtree)) (id u32) (w (box (dyn widget.widget))) (c (& ctx.ctx)))` | `unit` |
| `element.mount` | `((es (&mut elems)) (rt (&mut render.rtree)) (w (box (dyn widget.widget))))` | `u32` |
| `element.mark` | `((es (&mut elems)) (rt (&mut render.rtree)) (id u32))` | `unit` |
| `element.rebuild` | `((es (&mut elems)) (rt (&mut render.rtree)) (id u32) (c (& ctx.ctx)))` | `unit` |

## Tests

`element_reconcile`
