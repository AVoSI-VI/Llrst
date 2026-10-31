# std.math

Float math with no libm. f32 entry points round an f64 result.

Source: `std/math.llr`. Import with `(use std.math)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `math.abs` | `((x f64))` | `f64` |
| `math.min` | `((a f64) (b f64))` | `f64` |
| `math.max` | `((a f64) (b f64))` | `f64` |
| `math.floor` | `((x f64))` | `f64` |
| `math.ceil` | `((x f64))` | `f64` |
| `math.round` | `((x f64))` | `f64` |
| `math.sqrt` | `((x f64))` | `f64` |
| `math.abs32` | `((x f32))` | `f32` |
| `math.min32` | `((a f32) (b f32))` | `f32` |
| `math.max32` | `((a f32) (b f32))` | `f32` |
| `math.floor32` | `((x f32))` | `f32` |
| `math.ceil32` | `((x f32))` | `f32` |
| `math.round32` | `((x f32))` | `f32` |
| `math.sqrt32` | `((x f32))` | `f32` |

## Tests

`math_floor`, `math_sqrt`, `math_f32`
