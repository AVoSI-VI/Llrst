# ui.m_field

One-line field. It owns the string, the caret, and the selection.

Source: `ui/m_field.llr`. Import with `(use ui.m_field)` for module functions; types and methods need no import.

## Types

- `(struct edit (pub (text str)) (pub (at i64)) (pub (mark i64)) (pub (on bool)) (pub (send bool)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `m_field.edit_new` | `()` | `edit` |
| `m_field.on_input` | `((self (&mut edit)) (e (& state.ev)) (o (&mut state.out)))` | `bool` |
| `m_field.field` | `()` | `(box (dyn widget.widget))` |

## Tests

`field_edit`
