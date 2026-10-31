# ui.host_shim

Desktop shim calls. The linked object is the host operating system.

Source: `ui/host_shim.llr`. Import with `(use ui.host_shim)` for module functions; types and methods need no import.

## Types

- `(struct slot (p (ptr u8)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `host_shim.llr_ui_open` | `((w i64) (h i64))` | `(ptr u8)` |
| `host_shim.llr_ui_close` | `((p (ptr u8)))` | `unit` |
| `host_shim.llr_ui_size` | `((p (ptr u8)) (w (ptr i64)) (h (ptr i64)))` | `unit` |
| `host_shim.llr_ui_scale` | `((p (ptr u8)))` | `f32` |
| `host_shim.llr_ui_present` | `((p (ptr u8)) (px (ptr u8)) (w i64) (h i64) (stride i64))` | `unit` |
| `host_shim.llr_ui_poll` | `((p (ptr u8)) (buf (ptr u8)) (cap i64))` | `i64` |
| `host_shim.llr_ui_request_frame` | `((p (ptr u8)))` | `unit` |
| `host_shim.llr_ui_now_ms` | `((p (ptr u8)))` | `i64` |
| `host_shim.llr_ui_set_cursor` | `((p (ptr u8)) (id i64))` | `unit` |
| `host_shim.llr_ui_clip_get` | `((p (ptr u8)) (buf (ptr u8)) (cap i64))` | `i64` |
| `host_shim.llr_ui_clip_set` | `((p (ptr u8)) (bytes (ptr u8)) (n i64))` | `unit` |
| `host_shim.llr_ui_text_input` | `((p (ptr u8)) (on i64) (x f32) (y f32) (w f32) (h f32))` | `unit` |
| `host_shim.llr_ui_quit` | `((p (ptr u8)))` | `unit` |
| `host_shim.close` | `((s (&mut slot)))` | `unit` |
| `host_shim.quit` | `((s (&mut slot)))` | `unit` |
| `slot.drop` | `((self (&mut slot)))` | `unit` |
| `host_shim.open` | `((w i64) (h i64))` | `slot` |
| `host_shim.size` | `((s (& slot)))` | `geom.size` |
| `host_shim.scale` | `((s (& slot)))` | `f32` |
| `host_shim.present` | `((s (& slot)) (bm (& bitmap.bitmap)))` | `unit` |
| `host_shim.poll` | `((s (& slot)) (out (&mut (vec event.event))))` | `unit` |
| `host_shim.request_frame` | `((s (& slot)))` | `unit` |
| `host_shim.now_ms` | `((s (& slot)))` | `i64` |
| `host_shim.set_cursor` | `((s (& slot)) (id i64))` | `unit` |
| `host_shim.clip_get` | `((s (& slot)))` | `str` |
| `host_shim.clip_set` | `((s (& slot)) (text (& str)))` | `unit` |
| `host_shim.text_input` | `((s (& slot)) (on bool) (r (& geom.rect)))` | `unit` |
