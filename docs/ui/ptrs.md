# ui.ptrs

Pointer, focus, and animation tracks for one frame.

Source: `ui/ptrs.llr`. Import with `(use ui.ptrs)` for module functions; types and methods need no import.

## Types

- `(struct ptrs (pub (focus i64)) (pub (cap u32)) (pub (has_cap bool)) (pub (text str)) (pub (quit bool)) (pub (paused bool)) (pub (il f32)) (pub (it f32)) (pub (ir f32)) (pub (ib f32)) (pub (anims (vec anim.track))) (pub (sx f32)) (pub (sy f32)) (pub (lx f32)) (pub (ly f32)) (pub (drag bool)) (pub (ime bool)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `ptrs.ptrs_new` | `()` | `ptrs` |
