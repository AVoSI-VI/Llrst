# ui.sched

One frame's events: hit, messages, focus, then the caller rebuilds.

Source: `ui/sched.llr`. Import with `(use ui.sched)` for module functions; types and methods need no import.

## Types

- `(struct ptrs (pub (focus i64)) (pub (cap u32)) (pub (has_cap bool)) (pub (text str)) (pub (quit bool)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `sched.ptrs_new` | `()` | `ptrs` |
| `sched.handle` | `((es (&mut element.elems)) (rt (&mut render.rtree)) (root u32) (p (&mut ptrs)) (ev (& (vec event.event))))` | `unit` |
