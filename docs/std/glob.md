# std.glob

Glob matching and directory walking.

Source: `std/glob.llr`. Import with `(use std.glob)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `glob.matches` | `((pat (& str)) (p (& str)))` | `bool` |
| `glob.matches_path` | `((pat (& str)) (p (& str)))` | `bool` |
| `glob.walk` | `((dir (& str)))` | `(vec str)` |
| `glob.find` | `((dir (& str)) (pat (& str)))` | `(vec str)` |

## Tests

`glob_seg`
