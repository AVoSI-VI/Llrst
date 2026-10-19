# std.xml

One XML document. Elements and text. No attributes, namespaces, comments, declarations, or CDATA.

```
doc  := elem
elem := '<' name '>' node* '</' name '>' | '<' name '/>'
node := elem | text
```

Whitespace between tags is skipped. A text node is the bytes between tags. `<`, `>`, and `&` in text are `&lt;`, `&gt;`, and `&amp;`. Anything else in an entity, a mismatched end tag, a second root, or an attribute is `xml.parse`.

`(struct node (kind u8) (text str) (lo i64) (hi i64))` with kind 0 element and kind 1 text. The element name is `text`. Children of an element are an index range.

Source: `std/xml.llr`. Import with `(use std.xml)` for module functions; types and methods need no import.

## Types

- `(struct doc (nodes (vec node)) (kids (vec i64)) (pub (root i64)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `xml.read` | `((src (& str)))` | `(res doc str)` |
| `xml.kind` | `((d (& doc)) (n i64))` | `u8` |
| `xml.text` | `((d (& doc)) (n i64))` | `(& str)` |
| `xml.count` | `((d (& doc)) (n i64))` | `i64` |
| `xml.child` | `((d (& doc)) (n i64) (k i64))` | `i64` |
| `xml.escape` | `((s (& str)))` | `str` |
| `xml.tag` | `((name (& str)) (inner (& str)))` | `str` |
| `xml.empty` | `((name (& str)))` | `str` |

`xml.tag` and `xml.empty` do not escape `name`. `inner` is raw XML.

## Tests

`xml_tree`, `xml_escape`, `xml_parse`
