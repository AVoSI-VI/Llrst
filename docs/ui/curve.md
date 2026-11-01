# ui.curve

Unit curves. ease_in, ease_out, and ease_in_out are the CSS cubics.

Source: `ui/curve.llr`. Import with `(use ui.curve)` for module functions; types and methods need no import.

## Constants

| name | type | value |
|------|------|-------|
| `curve.linear` | `i64` | `0` |
| `curve.ease_in` | `i64` | `1` |
| `curve.ease_out` | `i64` | `2` |
| `curve.ease_in_out` | `i64` | `3` |

## Functions

| name | params | returns |
|------|--------|---------|
| `curve.bezier` | `((x1 f32) (y1 f32) (x2 f32) (y2 f32) (t f32))` | `f32` |
| `curve.apply` | `((id i64) (t f32))` | `f32` |

## Tests

`curve_ends`
