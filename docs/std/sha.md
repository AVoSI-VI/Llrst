# std.sha

SHA-1 and base64 for the WebSocket handshake. `sha.sha1` returns the raw 20-byte digest. `sha.b64` encodes those bytes.

Source: `std/sha.llr`. Import with `(use std.sha)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `sha.sha1` | `((msg (& str)))` | `str` |
| `sha.b64` | `((msg (& str)))` | `str` |

## Tests

`sha_empty`, `sha_accept`
