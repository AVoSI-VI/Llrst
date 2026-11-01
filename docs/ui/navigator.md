# ui.navigator

A stack of routes. Push slides the new page in; pop slides it back out.

Source: `ui/navigator.llr`. Import with `(use ui.navigator)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `navigator.navigator` | `((body (clos (i64) (box (dyn widget.widget)))))` | `(box (dyn widget.widget))` |

## Tests

`nav_pop`
