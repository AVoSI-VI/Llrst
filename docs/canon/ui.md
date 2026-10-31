# Canon: ui

Four ways to build a widget. `row`, `column`, and `stack` are framework layout.
A custom render object is a leaf: it has no children. `ctx.theme` is a background
and foreground pair until phase 11 adds `ui/theme.llr`.

Type ids `1` through `19` are the basic widgets. A user stateful id starts at `100`.
A closure body does not contain a `clos` form. It may call a function that creates one.

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
  (fn dispose ((_s (&mut model))) unit ()))

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
