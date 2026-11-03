# ui.sched

One frame's events: hit, messages, focus, then the caller rebuilds.

Source: `ui/sched.llr`. Import with `(use ui.sched)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `sched.handle` | `((es (&mut element.elems)) (rt (&mut render.rtree)) (root u32) (p (&mut ptrs.ptrs)) (host (&mut (box (dyn host.host)))) (ev (& (vec event.event))))` | `unit` |

## Tests

`sched_life`
