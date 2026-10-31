# ui.app

Opens a tree, runs frames, and dumps it. The first layout happens before events.

Source: `ui/app.llr`. Import with `(use ui.app)` for module functions; types and methods need no import.

## Types

- `(struct app (host (box (dyn host.host))) (es element.elems) (rt render.rtree) (g render_box.fonts) (root u32) (p sched.ptrs) (vw f32) (vh f32) (sc f32))`

## Functions

| name | params | returns |
|------|--------|---------|
| `app.open` | `((h (box (dyn host.host))) (root (box (dyn widget.widget))))` | `app` |
| `app.frame` | `((a (&mut app)))` | `unit` |
| `app.run` | `((h (box (dyn host.host))) (root (box (dyn widget.widget))) (frames i64))` | `unit` |
| `app.dump` | `((a (& app)))` | `str` |
