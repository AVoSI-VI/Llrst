# ui.w_theme

A theme for the descendants of one widget. The closure is called with that theme.

Source: `ui/w_theme.llr`. Import with `(use ui.w_theme)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `w_theme.wrap` | `((th ctx.palette) (dark bool) (bld (clos ((& ctx.ctx)) (box (dyn widget.widget)))))` | `(box (dyn widget.widget))` |
| `w_theme.light` | `((bld (clos ((& ctx.ctx)) (box (dyn widget.widget)))))` | `(box (dyn widget.widget))` |
| `w_theme.dark` | `((bld (clos ((& ctx.ctx)) (box (dyn widget.widget)))))` | `(box (dyn widget.widget))` |

## Tests

`w_theme_dark`
