# ui.hit

Deepest gesture whose rect contains the point. Later children are in front.

Source: `ui/hit.llr`. Import with `(use ui.hit)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `hit.at` | `((rt (& render.rtree)) (id u32) (x f32) (y f32))` | `(opt u32)` |

## Tests

`hit_empty`
