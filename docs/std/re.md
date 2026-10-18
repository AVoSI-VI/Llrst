# std.re

Backtracking regex: `. ^ $ * + ? {n,m} [..] [^..] \d \w \s \b ( ) |`. Captures are `mat.groups`.

Source: `std/re.llr`. Import with `(use std.re)` for module functions; types and methods need no import.

## Types

- `(struct re (alts (vec alt)))`
- `(struct grp (pub (lo i64)) (pub (hi i64)))`
- `(struct mat (pub (lo i64)) (pub (hi i64)) (pub (groups (vec grp))))`

## Functions

| name | params | returns |
|------|--------|---------|
| `re.compile` | `((pat (& str)))` | `(res re str)` |
| `re.boundary` | `((s (& str)) (i i64))` | `bool` |
| `re.find_from` | `((r (& re)) (s (& str)) (start i64))` | `(opt mat)` |
| `re.find` | `((r (& re)) (s (& str)))` | `(opt mat)` |
| `re.is_match` | `((r (& re)) (s (& str)))` | `bool` |
| `re.find_all` | `((r (& re)) (s (& str)))` | `(vec mat)` |
| `re.expand` | `((s (& str)) (m (& mat)) (new (& str)))` | `(res str str)` |

## Tests

`re_basic`, `re_find`, `re_cap`, `re_word`, `re_expand`
