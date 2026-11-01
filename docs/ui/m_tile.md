# ui.m_tile

List tile, card, divider, and a determinate progress bar.

Source: `ui/m_tile.llr`. Import with `(use ui.m_tile)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `m_tile.list_tile` | `((title str) (trail (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `m_tile.card` | `((child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `m_tile.divider` | `()` | `(box (dyn widget.widget))` |
| `m_tile.progress` | `((t f32))` | `(box (dyn widget.widget))` |

## Tests

`m_tile_name`
