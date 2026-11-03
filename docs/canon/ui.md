# Canon: ui

Four ways to build a widget, and one way to animate. `row`, `column`, `stack`,
and `scroll` are framework layout. A custom render object is a leaf: it has no
children. `ctx.theme` is a `ctx.palette` from `ui/theme.llr`.

Type ids `1` through `19` are the basic widgets. `20` is `animated_builder`, `21`
is `animated_container`, `22` is `animated_opacity`, and `23` is `animated_padding`.
`24` through `41` are the phase 11 set: `theme scroll list scaffold app_bar button
icon_button text_field checkbox switch slider dialog snack_bar list_tile card
divider progress navigator`. `42` is `semantics`. A user stateful id starts at `100`. A closure body
does not contain a `clos` form. It may call a function that creates one. A closure
must not move a captured box out of its environment; a widget that wraps a child
takes it through `widget.view_kids`, which passes the children as an argument.

## Stateless

A helper returns the child tree. The widget holds that function:

```
(widget.stateless k (clos ((c (& ctx.ctx))) (box (dyn widget.widget)) (view c)))
```

## Stateful

A struct implements `state.state`. `build` may call functions that create closures.
`msg` mutates `self` and returns `true` when it handled the id; `false` walks to
the parent state. The framework marks the element dirty after a handled `msg`.
A later update drops the new widget's init and keeps the element state. `input`
receives pointer, key, text, paste, submit, back, focus, and flush events through
`state.ev` and answers in `state.out`. `route` is the navigator depth, or `-1`.
`save` returns a `str` and `load` reads it back. `layout` is `(as i64 (sizeof Self))`.
A state that holds a `(clos ...)` or a `(fn ...)` returns `-1` from `layout`; a reload
then restarts and runs `save` then `load`, because those pointers would point into the
unloaded library.

```
(struct model (n i64))

(impl state.state model
  (fn build ((self (& model)) (_c (& ctx.ctx)) (_k (vec (box (dyn widget.widget))))) (box (dyn widget.widget))
    (view (. self n)))
  (fn msg ((self (&mut model)) (m i64)) bool
    (if (== m 1) (do (set (. self n) (+ (. self n) 1)) (ret true)))
    false)
  (fn input ((_s (&mut model)) (_e (& state.ev)) (_o (&mut state.out))) bool false)
  (fn route ((_s (& model))) i64 (neg 1))
  (fn dispose ((_s (&mut model))) unit ())
  (fn ticking ((_s (& model))) bool false)
  (fn tick ((_s (&mut model)) (_d i64) (_g (& anim.goal))) bool false)
  (fn save ((self (& model))) str (str.from_int (. self n)))
  (fn load ((self (&mut model)) (text (& str))) unit
    (match (str.to_int text) ((opt.some n) (set (. self n) n)) ((opt.none) ())))
  (fn layout ((_s (& model))) i64 (as i64 (sizeof model)))))

(state.stateful 100 (key.key.none) (model 0))
```

The third `build` argument is the child list given to `state.holding`; `scroll`
uses it and every other state ignores it.

## Callback

The closure returns a message id. `0` means none. Create it in a function or a
`state` method:

```
(w_gesture.gesture (clos () i64 1) child)
(w_gesture.tap 1 child)
```

## Custom render

Implement `render.render` with `layout`, `paint`, and `hit`, then box it. No children.

```
(widget.custom ty k r)
```

## Animation

A state that moves embeds `anim.ctrl`. `ticking` returns `anim.ctrl.running`.
`tick` calls `anim.ctrl.advance` with the elapsed milliseconds and returns true
when the value changed or the controller is still running. The frame runs that
before rebuild. Curves are `curve.linear`, `curve.ease_in`, `curve.ease_out`,
and `curve.ease_in_out`. `tween.num`, `tween.color`, `tween.rect`, and
`tween.size` interpolate. Color bytes are the stored premultiplied channels.

`w_anim.builder` owns a controller, starts it forward, and calls
`(clos ((t f32)) (box (dyn widget.widget)) ...)` with the curved value.
Messages `1`, `2`, `3`, and `4` are forward, reverse, repeat, and stop.

`w_anim.container`, `w_anim.opacity`, and `w_anim.padding` are leaves, so the
child stays mounted. The first frame shows the given target. A later target
tweens from the value on screen. `container` animates color and padding.
`navigator` runs a 200ms `ease_in_out` controller on a `transform` around the
top page: `tween.num` from `ctx.w` to `0` on push, reversed on pop.

## Theme

`ctx.theme` returns a `ctx.palette`: `bg surface primary on_primary fg muted line
danger`, text sizes `title` 20, `body` 16, `label` 14, and `space` 8. `theme.light`
and `theme.dark` are the two palettes. `w_theme.dark` (or `w_theme.light`) wraps a
subtree; the element reads the widget key (`1` is dark) and passes that palette
down through `ctx`. A `widget.view` reads the theme in its build closure, so the
dump names the control (`button`) rather than `stateless`.

## Input

`sched.handle` takes the host so clipboard and `text_input` run before the next
event. Pointer down focuses the hit gesture; a drag past 8px is captured by the
nearest `scroll` or `list` state and the tap is cancelled; under 8px the up still
taps. Wheel goes to the nearest state that accepts it. Keys and text go to the
focused state. `back` (record kind 8, script line `back`, Android `AKEYCODE_BACK`,
web `popstate`) goes to every state of type `navigator`. The web host calls
`history(depth)` after each tick; a growing depth is `pushState`, a shrinking one
is `history.back()` unless a `popstate` just caused it.

## One widget per task

| task | widget |
|------|--------|
| page frame with a title bar | `m_shell.scaffold` + `m_shell.app_bar` |
| run an action | `m_button.button` (`icon_button` for a glyph label) |
| one line of text entry | `m_field.field` (Enter sends `in_submit` to the parent state) |
| on/off | `m_toggle.checkbox`, `m_toggle.switch` (controlled: `on`, `msg`) |
| pick a number | `m_toggle.slider` (owns 0..100, drag sets it) |
| modal question | `m_layer.dialog` (scrim tap is the dismiss message) |
| short notice | `m_layer.snack_bar` (ticks out) |
| row in a list | `m_tile.list_tile`; group with `m_tile.card`; split with `m_tile.divider` |
| long content | `w_scroll.scroll`; many fixed rows: `w_list.list` (mounts the visible window only) |
| progress | `m_tile.progress` |
| pages | `navigator.navigator`; message `1` pops, `100 + i` pushes route `i` |

A page closure on `navigator` is captured once at mount, so a page that shows
model data should be built by the model's own state: `examples/todo.llr` keeps
the route in its state with type id `widget.ty_nav`, which makes `back` and the
web history reach it.
