# std.re

Backtracking regex: `. ^ $ * + ? {n,m} [..] [^..] \d \w \s ( ) |`.

Source: `std/re.llr`. Import with `(use std.re)` for module functions; types and methods need no import.

## Types

- `(struct atom (kind u8) (c u8) (set str) (neg bool) (grp i64) (min i64) (max i64))`
- `(struct seq (atoms (vec atom)))`
- `(struct alt (seqs (vec seq)))`
- `(struct re (alts (vec alt)))`
- `(struct mat (lo i64) (hi i64))`
- `(struct parser (p str) (i i64) (alts (vec alt)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `re.compile` | `((pat (& str)))` | `(res re str)` |
| `re.find_from` | `((r (& re)) (s (& str)) (start i64))` | `(opt mat)` |
| `re.find` | `((r (& re)) (s (& str)))` | `(opt mat)` |
| `re.is_match` | `((r (& re)) (s (& str)))` | `bool` |
| `re.find_all` | `((r (& re)) (s (& str)))` | `(vec mat)` |

## Tests

`re_basic`, `re_find`
