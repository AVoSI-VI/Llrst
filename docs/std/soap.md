# std.soap

One SOAP call. The body is an XML envelope. The result is an XML envelope. [`xml`](xml.md) parses and prints it. A body is one operation: no namespaces, no attributes, and no nested elements past the arguments.

`hello` is `world`. `ping` is `pong`. `add` sums the integer text of `a` and `b`. A missing argument is 0. A good call is `<Envelope><Body><name><result>VALUE</result></name></Body></Envelope>`. A bad envelope is `<Envelope><Body><Fault>soap.parse</Fault></Body></Envelope>`. An unknown operation is `<Envelope><Body><Fault>soap.op|name=</Fault></Body></Envelope>`.

Source: `std/soap.llr`. Import with `(use std.soap)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `soap.exec` | `((q (& str)))` | `str` |

## Tests

`soap_fields`, `soap_ops`, `soap_http`
