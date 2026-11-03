# ui.focus

Tab order is the preorder of gesture elements.

Source: `ui/focus.llr`. Import with `(use ui.focus)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `focus.collect` | `((es (& element.elems)) (rt (& render.rtree)) (id u32) (out (&mut (vec u32))))` | `unit` |
| `focus.tab` | `((ids (& (vec u32))) (cur u32) (has bool) (shift bool))` | `(opt u32)` |

## Tests

`focus_tab`
