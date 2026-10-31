# ui.widget

Widget trait, leaf descriptors, and stateless. Type ids stand in for a runtime type.

Source: `ui/widget.llr`. Import with `(use ui.widget)` for module functions; types and methods need no import.

## Types

- `(struct rdesc (pub (pad_l f32)) (pub (pad_t f32)) (pub (pad_r f32)) (pub (pad_b f32)) (pub (ax f32)) (pub (ay f32)) (pub (has_color bool)) (pub (color blend.color)) (pub (text str)) (pub (font_size f32)) (pub (has_image bool)) (pub (image bitmap.bitmap)) (pub (opacity f32)) (pub (clip bool)) (pub (has_xform bool)) (pub (tx f32)) (pub (ty f32)) (pub (sx f32)) (pub (sy f32)) (pub (rot f32)) (pub (min_w f32)) (pub (max_w f32)) (pub (min_h f32)) (pub (max_h f32)) (pub (tight bool)) (pub (has_custom bool)) (pub (custom (box (dyn render.render)))) (pub (flex i64)) (pub (has_pos bool)) (pub (px f32)) (pub (py f32)) (pub (pw f32)) (pub (ph f32)))`
- `(struct pdata (pub (flex i64)) (pub (has_pos bool)) (pub (x f32)) (pub (y f32)) (pub (w f32)) (pub (h f32)) (pub (gesture bool)) (pub (relayout bool)) (pub (repaint bool)))`
- `(struct leaf (ty i64) (k key.key) (kids (vec (box (dyn widget)))) (gesture bool) (tap (clos () i64)) (d rdesc))`

## Traits

| name | params | returns |
|------|--------|---------|
| `widget.ty` | `((& Self))` | `i64` |
| `widget.name` | `((& Self))` | `(& str)` |
| `widget.key` | `((& Self))` | `key.key` |
| `widget.role` | `((& Self))` | `i64` |
| `widget.set_key` | `((&mut Self) key.key)` | `unit` |
| `widget.build` | `((&mut Self) (& ctx.ctx))` | `(box (dyn widget))` |
| `widget.take_state` | `((&mut Self))` | `(opt (box (dyn state.state)))` |
| `widget.take_children` | `((&mut Self))` | `(vec (box (dyn widget)))` |
| `widget.fill` | `((&mut Self) (&mut rdesc))` | `unit` |
| `widget.pdata` | `((& Self))` | `pdata` |
| `widget.signal` | `((& Self))` | `i64` |

## Constants

| name | type | value |
|------|------|-------|
| `widget.ty_empty` | `i64` | `1` |
| `widget.ty_stateless` | `i64` | `2` |
| `widget.ty_container` | `i64` | `3` |
| `widget.ty_padding` | `i64` | `4` |
| `widget.ty_sized` | `i64` | `5` |
| `widget.ty_align` | `i64` | `6` |
| `widget.ty_center` | `i64` | `7` |
| `widget.ty_row` | `i64` | `8` |
| `widget.ty_column` | `i64` | `9` |
| `widget.ty_expanded` | `i64` | `10` |
| `widget.ty_flex` | `i64` | `11` |
| `widget.ty_stack` | `i64` | `12` |
| `widget.ty_positioned` | `i64` | `13` |
| `widget.ty_text` | `i64` | `14` |
| `widget.ty_image` | `i64` | `15` |
| `widget.ty_gesture` | `i64` | `16` |
| `widget.ty_opacity` | `i64` | `17` |
| `widget.ty_clip` | `i64` | `18` |
| `widget.ty_transform` | `i64` | `19` |
| `widget.role_leaf` | `i64` | `0` |
| `widget.role_less` | `i64` | `1` |
| `widget.role_full` | `i64` | `2` |

## Functions

| name | params | returns |
|------|--------|---------|
| `widget.no_data` | `()` | `pdata` |
| `widget.parent_data` | `((ty i64))` | `bool` |
| `widget.name_of` | `((ty i64))` | `(& str)` |
| `widget.nop` | `()` | `(clos () i64)` |
| `widget.blank` | `()` | `rdesc` |
| `widget.empty` | `()` | `(box (dyn widget))` |
| `widget.make` | `((ty i64) (k key.key) (kids (vec (box (dyn widget)))) (d rdesc) (gesture bool) (tap (clos () i64)))` | `(box (dyn widget))` |
| `widget.stateless` | `((k key.key) (bld (clos ((& ctx.ctx)) (box (dyn widget)))))` | `(box (dyn widget))` |
| `widget.with_key` | `((w (box (dyn widget))) (k key.key))` | `(box (dyn widget))` |
| `widget.custom` | `((ty i64) (k key.key) (r (box (dyn render.render))))` | `(box (dyn widget))` |

## Tests

`widget_name`
