# ui.host_web

Browser host. The wasm imports live in module `llr`. A native stub satisfies the link.

Source: `ui/host_web.llr`. Import with `(use ui.host_web)` for module functions; types and methods need no import.

## Types

- `(struct host_web (on bool))`

## Functions

| name | params | returns |
|------|--------|---------|
| `host_web.make` | `()` | `host_web` |
| `host_web.run` | `((root (box (dyn widget.widget))))` | `unit` |
| `host_web.llr_frame` | `()` | `unit` |
