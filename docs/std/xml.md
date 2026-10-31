# std.xml

One XML tree. Elements, text, attributes, comments, CDATA, processing instructions, and an XML declaration. Namespaces bind with `xmlns` and `xmlns:prefix` and are inherited. `(xml.ns)` is the URI. `(xml.local)` is the name after one colon. An unbound prefix or a `<!DOCTYPE` is `xml.parse`. Entities are `lt`, `gt`, `amp`, `quot`, `apos`, and numeric character references. Attributes sit beside content children.

Source: `std/xml.llr`. Import with `(use std.xml)` for module functions; types and methods need no import.

## Types

- `(struct doc (nodes (vec node)) (kids (vec i64)) (attrs (vec attr)) (pub (root i64)) (pub (version str)) (pub (encoding str)) (pub (standalone str)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `xml.read` | `((src (& str)))` | `(res doc str)` |
| `xml.kind` | `((d (& doc)) (n i64))` | `u8` |
| `xml.text` | `((d (& doc)) (n i64))` | `(& str)` |
| `xml.ns` | `((d (& doc)) (n i64))` | `(& str)` |
| `xml.local` | `((d (& doc)) (n i64))` | `str` |
| `xml.count` | `((d (& doc)) (n i64))` | `i64` |
| `xml.child` | `((d (& doc)) (n i64) (k i64))` | `i64` |
| `xml.attr_count` | `((d (& doc)) (n i64))` | `i64` |
| `xml.attr_name` | `((d (& doc)) (n i64) (k i64))` | `(& str)` |
| `xml.attr_value` | `((d (& doc)) (n i64) (k i64))` | `(& str)` |
| `xml.escape` | `((s (& str)))` | `str` |
| `xml.tag` | `((name (& str)) (inner (& str)))` | `str` |
| `xml.empty` | `((name (& str)))` | `str` |

## Tests

`xml_tree`, `xml_ns`, `xml_escape`, `xml_parse`, `xml_scan`
