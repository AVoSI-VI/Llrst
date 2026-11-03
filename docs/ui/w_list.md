# ui.w_list

Fixed-extent list. Only the rows in the host window are mounted.

Source: `ui/w_list.llr`. Import with `(use ui.w_list)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `w_list.window` | `((off f32) (extent f32) (h f32) (count i64) (lo (&mut i64)) (hi (&mut i64)))` | `unit` |
| `w_list.list` | `((extent f32) (count i64) (body (clos (i64) (box (dyn widget.widget)))))` | `(box (dyn widget.widget))` |

## Tests

`w_list_window`
