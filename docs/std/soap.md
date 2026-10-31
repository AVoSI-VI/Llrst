# std.soap

SOAP 1.1. The envelope namespace is `http://schemas.xmlsoap.org/soap/envelope/`. An empty namespace is also accepted. A `Header` block with `mustUnderstand="1"` is a Fault. The body holds one operation. `add` finds `a` and `b` by local name at any depth. The reply is an envelope with `<result>`. A bad call is `faultcode` and `faultstring`.

Source: `std/soap.llr`. Import with `(use std.soap)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `soap.exec` | `((q (& str)))` | `str` |

## Tests

`soap_fields`, `soap_ops`, `soap_http`
