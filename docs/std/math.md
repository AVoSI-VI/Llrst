# std.math

Float math with no libm. f32 entry points round an f64 result.

Source: `std/math.llr`. Import with `(use std.math)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `math.nan` | `()` | `f64` |
| `math.inf` | `()` | `f64` |
| `math.abs` | `((x f64))` | `f64` |
| `math.min` | `((a f64) (b f64))` | `f64` |
| `math.max` | `((a f64) (b f64))` | `f64` |
| `math.floor` | `((x f64))` | `f64` |
| `math.ceil` | `((x f64))` | `f64` |
| `math.round` | `((x f64))` | `f64` |
| `math.pow2` | `((e i64))` | `f64` |
| `math.sqrt` | `((x f64))` | `f64` |
| `math.abs32` | `((x f32))` | `f32` |
| `math.min32` | `((a f32) (b f32))` | `f32` |
| `math.max32` | `((a f32) (b f32))` | `f32` |
| `math.floor32` | `((x f32))` | `f32` |
| `math.ceil32` | `((x f32))` | `f32` |
| `math.round32` | `((x f32))` | `f32` |
| `math.sqrt32` | `((x f32))` | `f32` |
| `math.approx` | `((g f64) (w f64))` | `bool` |
| `math.imin` | `((a i64) (b i64))` | `i64` |
| `math.imax` | `((a i64) (b i64))` | `i64` |
| `math.iclamp` | `((x i64) (lo i64) (hi i64))` | `i64` |
| `math.iabs` | `((n i64))` | `i64` |
| `math.ipow` | `((x i64) (n i64))` | `i64` |
| `math.align_up` | `((x i64) (a i64))` | `i64` |
| `math.milli` | `((x f32))` | `i64` |
| `math.clamp` | `((x f64) (lo f64) (hi f64))` | `f64` |
| `math.clamp32` | `((x f32) (lo f32) (hi f32))` | `f32` |
| `math.lerp` | `((a f64) (b f64) (t f64))` | `f64` |
| `math.lerp32` | `((a f32) (b f32) (t f32))` | `f32` |
| `math.fmod` | `((x f64) (y f64))` | `f64` |
| `math.fmod32` | `((x f32) (y f32))` | `f32` |

## Tests

`math_floor`, `math_sqrt`, `math_f32`, `math_int`, `math_float_more`
