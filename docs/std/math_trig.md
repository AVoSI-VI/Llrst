# std.math_trig

sin, cos, atan2, and pow. f32 entry points round an f64 result.

Source: `std/math_trig.llr`. Import with `(use std.math_trig)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `math_trig.sin` | `((x f64))` | `f64` |
| `math_trig.cos` | `((x f64))` | `f64` |
| `math_trig.atan2` | `((y f64) (x f64))` | `f64` |
| `math_trig.pow` | `((x f64) (y f64))` | `f64` |
| `math_trig.sin32` | `((x f32))` | `f32` |
| `math_trig.cos32` | `((x f32))` | `f32` |
| `math_trig.atan232` | `((y f32) (x f32))` | `f32` |
| `math_trig.pow32` | `((x f32) (y f32))` | `f32` |

## Tests

`trig_known`, `trig_parts`
