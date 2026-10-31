# Desktop hosts

Win32, X11, and Cocoa present the same BGRA8 buffer and pull events into one 64-byte
record. The suite stays headless. `examples/counter.llr` keeps `host_null`. A window is:

```
(use ui.app)
(use ui.host_pick)
(app.run (host_pick.open 800 600) (root) 1000000)
```

`host_pick.which` is `2` on Windows, `3` on Darwin, and `1` on other POSIX. `open` returns
`(box (dyn host.host))`. A failed display panics `host.open`.

The link line adds the shim only when the object references `llr_ui_open`. Windows links
`user32 gdi32 imm32 shell32`. Darwin links `-framework Cocoa`. Other Unix links `-lX11`.
Install compiles the host shim: `clang -fobjc-arc` for the `.m` files on Darwin, `cc` for
the X11 files on Linux (`libx11-dev`), and `cl` for the Win32 files. Wayland is later.

## Record

Little-endian, 64 bytes. `event.take_rec` turns a buffer of them into `event` values and
joins adjacent text records into one `text` event.

| offset | field |
|--------|--------|
| 0 | `i32` kind: 1 pointer, 2 key, 3 text, 4 frame, 5 quit |
| 4 | `i32` a: pointer kind, key down (1 or 0), or text byte count |
| 8 | `i32` b: key code, a Unicode scalar (tab is 9) |
| 12 | `i32` c: modifiers, bit 0 shift, bit 1 ctrl, bit 2 alt, bit 3 meta |
| 16 | `f32` x y z w, logical pixels; scroll uses z and w |
| 32 | 32 UTF-8 bytes |

Pointer kind matches `event.down` through `event.scroll`. A positive scroll `w` is away
from the user. Cursor ids are `host.cursor_arrow` `cursor_text` `cursor_hand` `cursor_cross`
`cursor_wait`. Cocoa has no public wait cursor, so id 4 is the arrow there.

`poll` blocks until input, resize, a DPI change, close, or a previous `request_frame`.
The first poll returns immediately so the first frame paints. Close enqueues `quit`. The
app then calls `host.quit`.

## Manual check

Run a windowed counter on each desktop. The golden suite does not open a window.

- Resize the window. The next frame uses the new logical size.
- Change DPI or move the window to another monitor. `scale` follows the monitor, and the
  bitmap is in device pixels.
- Minimise, then restore. The window presents again.
- Close the window. The process exits after `quit`.
- Copy text in another program and paste through `clipboard_get`. `clipboard_set` is
  readable by another program.
- Press Tab. The key code is 9. Shift is modifier bit 0.
- With text input on, commit an IME string. It arrives as one `text` event.
- `set_cursor` switches arrow, text, hand, and cross.
