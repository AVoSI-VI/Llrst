# ui.state

Stateful widgets. The element keeps the first state; a later widget's init is dropped.

Source: `ui/state.llr`. Import with `(use ui.state)` for module functions; types and methods need no import.

## Types

- `(struct ev (pub (kind i64)) (pub (code i64)) (pub (mods i64)) (pub (x f32)) (pub (y f32)) (pub (dx f32)) (pub (dy f32)) (pub (w f32)) (pub (h f32)) (pub (text str)))`
- `(struct out (pub (do_copy bool)) (pub (do_paste bool)) (pub (submit bool)) (pub (ime bool)) (pub (handoff bool)) (pub (text str)) (pub (caret geom.rect)))`
- `(struct nostate)`

## Traits

| name | params | returns |
|------|--------|---------|
| `state.build` | `((& Self) (& ctx.ctx) (vec (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `state.msg` | `((&mut Self) i64)` | `bool` |
| `state.input` | `((&mut Self) (& ev) (&mut out))` | `bool` |
| `state.route` | `((& Self))` | `i64` |
| `state.dispose` | `((&mut Self))` | `unit` |
| `state.ticking` | `((& Self))` | `bool` |
| `state.tick` | `((&mut Self) i64 (& anim.goal))` | `bool` |

## Constants

| name | type | value |
|------|------|-------|
| `state.in_ptr` | `i64` | `1` |
| `state.in_key` | `i64` | `2` |
| `state.in_text` | `i64` | `3` |
| `state.in_paste` | `i64` | `4` |
| `state.in_submit` | `i64` | `5` |
| `state.in_back` | `i64` | `6` |
| `state.in_focus` | `i64` | `7` |
| `state.in_flush` | `i64` | `8` |

## Functions

| name | params | returns |
|------|--------|---------|
| `state.ev_new` | `()` | `ev` |
| `state.ev_ptr` | `((code i64) (x f32) (y f32) (dy f32) (w f32) (h f32))` | `ev` |
| `state.ev_text` | `((kind i64) (text str))` | `ev` |
| `state.out_new` | `()` | `out` |
| `state.no_state` | `()` | `(box (dyn state))` |
| `state.stateful` | `((ty i64) (k key.key) (s (box (dyn state))))` | `(box (dyn widget.widget))` |
| `state.holding` | `((ty i64) (k key.key) (s (box (dyn state))) (kids (vec (box (dyn widget.widget)))))` | `(box (dyn widget.widget))` |
