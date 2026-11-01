# Canon: ui

Four ways to build a widget, and one way to animate. `row`, `column`, and `stack`
are framework layout. A custom render object is a leaf: it has no children.
`ctx.theme` is a background and foreground pair until phase 11 adds `ui/theme.llr`.

Type ids `1` through `19` are the basic widgets. `20` is `animated_builder`, `21`
is `animated_container`, `22` is `animated_opacity`, and `23` is `animated_padding`.
A user stateful id starts at `100`. A closure body does not contain a `clos` form.
It may call a function that creates one.

## Stateless

A helper returns the child tree. The widget holds that function:

```
(widget.stateless k (clos ((c (& ctx.ctx))) (box (dyn widget.widget)) (view c)))
```

## Stateful

A struct implements `state.state`. `build` may call functions that create closures.
`msg` mutates `self`. The framework marks the element dirty after `msg`. A later
update drops the new widget's init and keeps the element state.

```
(struct model (n i64))

(impl state.state model
  (fn build ((self (& model)) (_c (& ctx.ctx))) (box (dyn widget.widget)) (view (. self n)))
  (fn msg ((self (&mut model)) (m i64)) unit
    (if (== m 1) (set (. self n) (+ (. self n) 1))))
  (fn dispose ((_s (&mut model))) unit ())
  (fn ticking ((_s (& model))) bool false)
  (fn tick ((_s (&mut model)) (_d i64) (_g (& anim.goal))) bool false))

(state.stateful 100 (key.key.none) (model 0))
```

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
The page transition that uses these controllers belongs to phase 11's navigator.
