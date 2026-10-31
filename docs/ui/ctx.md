# ui.ctx

Build snapshot. size is the host, not the element. Ancestors are nearest first.

Source: `ui/ctx.llr`. Import with `(use ui.ctx)` for module functions; types and methods need no import.

## Types

- `(struct theme (pub (bg blend.color)) (pub (fg blend.color)))`
- `(struct ctx (pub (w f32)) (pub (h f32)) (pub (scale f32)) (th theme) (pub (id u32)) (anc (vec u32)) (tys (vec i64)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `theme.standard` | `()` | `theme` |
| `ctx.root` | `((w f32) (h f32) (scale f32) (id u32))` | `ctx` |
| `ctx.size` | `((self (& ctx)))` | `geom.size` |
| `ctx.theme` | `((self (& ctx)))` | `theme` |
| `ctx.find_ancestor` | `((self (& ctx)) (ty i64))` | `(opt u32)` |
| `ctx.child` | `((self (& ctx)) (id u32) (parent_ty i64))` | `ctx` |

## Tests

`ctx_ancestor`
