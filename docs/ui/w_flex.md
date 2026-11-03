# ui.w_flex

Row, column, expanded, and flex. Flex factor is parent data on the child render node.

Source: `ui/w_flex.llr`. Import with `(use ui.w_flex)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `w_flex.row` | `((kids (vec (box (dyn widget.widget)))))` | `(box (dyn widget.widget))` |
| `w_flex.column` | `((kids (vec (box (dyn widget.widget)))))` | `(box (dyn widget.widget))` |
| `w_flex.flex` | `((n i64) (child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |
| `w_flex.expanded` | `((child (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |

## Tests

`w_flex_ty`
