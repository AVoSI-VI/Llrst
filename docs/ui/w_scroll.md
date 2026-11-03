# ui.w_scroll

A viewport that clips its child and shifts it by the scroll offset.

Source: `ui/w_scroll.llr`. Import with `(use ui.w_scroll)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `w_scroll.leaf` | `((off f32) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_scroll.scroll` | `((child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_scroll.metrics` | `((es (& element.elems)) (rt (& render.rtree)) (id u32) (view (&mut f32)) (span (&mut f32)))` | `unit` |

## Tests

`w_scroll_name`
