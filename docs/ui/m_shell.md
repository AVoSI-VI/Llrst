# ui.m_shell

Scaffold and app bar. The bar is a surface strip; the body takes the rest.

Source: `ui/m_shell.llr`. Import with `(use ui.m_shell)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `m_shell.app_bar` | `((title str))` | `(box (dyn widget.widget))` |
| `m_shell.scaffold` | `((bar (box (dyn widget.widget))) (body (box (dyn widget.widget))))` | `(box (dyn widget.widget))` |

## Tests

`m_shell_name`
