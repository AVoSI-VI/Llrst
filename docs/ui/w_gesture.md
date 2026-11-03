# ui.w_gesture

A tap target. The closure returns a message id; 0 means none.

Source: `ui/w_gesture.llr`. Import with `(use ui.w_gesture)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `w_gesture.gesture` | `((tap (clos () i64)) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_gesture.tap` | `((msg i64) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |

## Tests

`w_gesture_msg`
