# ui.layout

Pure flex and stack numbers. A main axis larger than unbound is unbounded.

Source: `ui/layout.llr`. Import with `(use ui.layout)` for module functions; types and methods need no import.

## Constants

| name | type | value |
|------|------|-------|
| `layout.unbound` | `f32` | `10000000000.0f32` |

## Functions

| name | params | returns |
|------|--------|---------|
| `layout.bounded` | `((x f32))` | `bool` |
| `layout.clamp` | `((x f32) (lo f32) (hi f32))` | `f32` |
| `layout.align_off` | `((space f32) (child f32) (a f32))` | `f32` |
| `layout.share` | `((free f32) (flex i64) (total i64))` | `f32` |
| `layout.free_main` | `((max f32) (used f32))` | `f32` |
| `layout.span` | `((max f32) (used f32) (lo f32))` | `f32` |

## Tests

`layout_share`
