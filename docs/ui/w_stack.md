# ui.w_stack

Stack and positioned. A negative width or height means the child chooses that axis.

Source: `ui/w_stack.llr`. Import with `(use ui.w_stack)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `w_stack.stack` | `((kids (vec (box (dyn widget.widget)))))` | `(box (dyn widget.widget))` |
| `w_stack.positioned` | `((x f32) (y f32) (w f32) (h f32) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |

## Tests

`w_stack_ty`
