# ui.state

Stateful widgets. The element keeps the first state; a later widget's init is dropped.

Source: `ui/state.llr`. Import with `(use ui.state)` for module functions; types and methods need no import.

## Types

- `(struct nostate)`

## Traits

| name | params | returns |
|------|--------|---------|
| `state.build` | `((& Self) (& ctx.ctx))` | `(box (dyn widget.widget))` |
| `state.msg` | `((&mut Self) i64)` | `unit` |
| `state.dispose` | `((&mut Self))` | `unit` |
| `state.ticking` | `((& Self))` | `bool` |
| `state.tick` | `((&mut Self) i64 (& anim.goal))` | `bool` |

## Functions

| name | params | returns |
|------|--------|---------|
| `state.no_state` | `()` | `(box (dyn state))` |
| `state.stateful` | `((ty i64) (k key.key) (s (box (dyn state))))` | `(box (dyn widget.widget))` |
