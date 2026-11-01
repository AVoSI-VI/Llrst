# ui.anim

Controller, implicit target, and the track a ticker stores.

Source: `ui/anim.llr`. Import with `(use ui.anim)` for module functions; types and methods need no import.

## Types

- `(struct ctrl (pub (dur i64)) (pub (pos i64)) (pub (dir i64)) (pub (rep i64)) (pub (status i64)))`
- `(struct goal (pub (on bool)) (pub (mask i64)) (pub (color blend.color)) (pub (l f32)) (pub (t f32)) (pub (r f32)) (pub (b f32)) (pub (w f32)) (pub (h f32)) (pub (opacity f32)) (pub (dur i64)) (pub (curve i64)))`
- `(struct track (pub (elem u32)) (pub (ctrl ctrl)) (pub (from goal)) (pub (to goal)) (pub (cur goal)))`

## Constants

| name | type | value |
|------|------|-------|
| `anim.dismissed` | `i64` | `0` |
| `anim.forward` | `i64` | `1` |
| `anim.reverse` | `i64` | `2` |
| `anim.completed` | `i64` | `3` |
| `anim.bit_color` | `i64` | `1` |
| `anim.bit_pad` | `i64` | `2` |
| `anim.bit_size` | `i64` | `4` |
| `anim.bit_opacity` | `i64` | `8` |
| `anim.rep_none` | `i64` | `0` |
| `anim.rep_restart` | `i64` | `1` |
| `anim.rep_reverse` | `i64` | `2` |

## Functions

| name | params | returns |
|------|--------|---------|
| `ctrl.make` | `((dur i64))` | `ctrl` |
| `ctrl.value` | `((self (& ctrl)))` | `f32` |
| `ctrl.running` | `((self (& ctrl)))` | `bool` |
| `ctrl.status` | `((self (& ctrl)))` | `i64` |
| `ctrl.forward` | `((self (&mut ctrl)))` | `unit` |
| `ctrl.reverse` | `((self (&mut ctrl)))` | `unit` |
| `ctrl.repeat` | `((self (&mut ctrl)) (rev bool))` | `unit` |
| `ctrl.stop` | `((self (&mut ctrl)))` | `unit` |
| `ctrl.advance` | `((self (&mut ctrl)) (dt i64))` | `bool` |
| `anim.goal_new` | `()` | `goal` |
| `anim.copy_goal` | `((g (& goal)))` | `goal` |
| `anim.same` | `((a (& goal)) (b (& goal)))` | `bool` |
| `anim.track_make` | `((elem u32) (g (& goal)))` | `track` |
| `anim.track_sync` | `((tr (&mut track)) (g (& goal)))` | `bool` |
| `anim.track_show` | `((tr (&mut track)) (skip bool) (dt i64))` | `bool` |

## Tests

`anim_drive`, `anim_track`
