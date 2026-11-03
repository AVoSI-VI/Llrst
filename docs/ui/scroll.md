# ui.scroll

Vertical offset. Clamped to the content, with a fling that dies at the edge.

Source: `ui/scroll.llr`. Import with `(use ui.scroll)` for module functions; types and methods need no import.

## Types

- `(struct step (pub (off f32)) (pub (vel f32)))`

## Constants

| name | type | value |
|------|------|-------|
| `scroll.slop` | `f32` | `8.0f32` |

## Functions

| name | params | returns |
|------|--------|---------|
| `scroll.limit` | `((span f32) (view f32))` | `f32` |
| `scroll.clamp` | `((off f32) (lim f32))` | `f32` |
| `scroll.drag` | `((off f32) (dy f32) (lim f32))` | `f32` |
| `scroll.wheel` | `((off f32) (dy f32) (lim f32))` | `f32` |
| `scroll.fling_step` | `((off f32) (vel f32) (lim f32))` | `step` |
| `scroll.fling` | `((off f32) (vel f32) (lim f32) (dt i64))` | `step` |
| `scroll.live` | `((vel f32) (drag bool))` | `bool` |
| `scroll.touch` | `((off (&mut f32)) (vel (&mut f32)) (held (&mut bool)) (code i64) (dy f32) (view f32) (span f32))` | `bool` |
| `scroll.coast` | `((off (&mut f32)) (vel (&mut f32)) (drag bool) (view f32) (span f32) (dt i64))` | `bool` |

## Tests

`scroll_clamp`
