# ui.ticker

Per-frame ticks. States run before rebuild; implicit tracks write the render desc after.

Source: `ui/ticker.llr`. Import with `(use ui.ticker)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `ticker.write_desc` | `((d (&mut widget.rdesc)) (g (& anim.goal)))` | `unit` |
| `ticker.pump_states` | `((es (&mut element.elems)) (rt (&mut render.rtree)) (dt i64))` | `bool` |
| `ticker.pump_implicit` | `((es (& element.elems)) (rt (&mut render.rtree)) (p (&mut sched.ptrs)) (dt i64))` | `bool` |

## Tests

`ticker_pump`
