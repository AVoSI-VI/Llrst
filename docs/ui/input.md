# ui.input

Pointer drags, wheels, keys, text, and clipboard. A false message walks upward.

Source: `ui/input.llr`. Import with `(use ui.input)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `input.bubble` | `((es (&mut element.elems)) (rt (&mut render.rtree)) (id u32))` | `unit` |
| `input.focus_to` | `((es (&mut element.elems)) (rt (&mut render.rtree)) (host (&mut (box (dyn host.host)))) (p (&mut ptrs.ptrs)) (id u32))` | `unit` |
| `input.pointer` | `((es (&mut element.elems)) (rt (&mut render.rtree)) (host (&mut (box (dyn host.host)))) (p (&mut ptrs.ptrs)) (hit (opt u32)) (kind i64) (x f32) (y f32) (dy f32))` | `unit` |
| `input.key` | `((es (&mut element.elems)) (rt (&mut render.rtree)) (host (&mut (box (dyn host.host)))) (p (&mut ptrs.ptrs)) (code i64) (mods i64))` | `unit` |
| `input.text` | `((es (&mut element.elems)) (rt (&mut render.rtree)) (host (&mut (box (dyn host.host)))) (p (&mut ptrs.ptrs)) (s (& str)))` | `unit` |
| `input.back` | `((es (&mut element.elems)) (rt (&mut render.rtree)) (host (&mut (box (dyn host.host)))) (p (&mut ptrs.ptrs)))` | `unit` |
| `input.flush` | `((es (&mut element.elems)) (rt (&mut render.rtree)) (host (&mut (box (dyn host.host)))) (p (&mut ptrs.ptrs)))` | `unit` |
