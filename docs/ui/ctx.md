# ui.ctx

Build snapshot. size is the host, not the element. Ancestors are nearest first.

Source: `ui/ctx.llr`. Import with `(use ui.ctx)` for module functions; types and methods need no import.

## Types

- `(struct palette (pub (bg blend.color)) (pub (surface blend.color)) (pub (primary blend.color)) (pub (on_primary blend.color)) (pub (fg blend.color)) (pub (muted blend.color)) (pub (line blend.color)) (pub (danger blend.color)) (pub (title f32)) (pub (body f32)) (pub (label f32)) (pub (space f32)))`
- `(struct ctx (pub (w f32)) (pub (h f32)) (pub (scale f32)) (th palette) (pub (id u32)) (anc (vec u32)) (tys (vec i64)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `ctx.copy_pal` | `((t (& palette)))` | `palette` |
| `ctx.palette_light` | `()` | `palette` |
| `ctx.palette_dark` | `()` | `palette` |
| `ctx.root` | `((w f32) (h f32) (scale f32) (id u32))` | `ctx` |
| `ctx.size` | `((self (& ctx)))` | `geom.size` |
| `ctx.theme` | `((self (& ctx)))` | `palette` |
| `ctx.with_theme` | `((self (& ctx)) (th palette))` | `ctx` |
| `ctx.find_ancestor` | `((self (& ctx)) (ty i64))` | `(opt u32)` |
| `ctx.child` | `((self (& ctx)) (id u32) (parent_ty i64))` | `ctx` |

## Tests

`ctx_ancestor`
