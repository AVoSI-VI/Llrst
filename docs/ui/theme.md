# ui.theme

Light and dark palettes. Descendants read them through ctx.theme.

Source: `ui/theme.llr`. Import with `(use ui.theme)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `theme.ink` | `((c (& blend.color)))` | `blend.color` |
| `theme.light` | `()` | `ctx.palette` |
| `theme.dark` | `()` | `ctx.palette` |
| `theme.copy` | `((t (& ctx.palette)))` | `ctx.palette` |

## Tests

`theme_pair`
