# std.cli

Argument parsing for tools.

Source: `std/cli.llr`. Import with `(use std.cli)` for module functions; types and methods need no import.

## Types

- `(struct cli)`

## Functions

| name | params | returns |
|------|--------|---------|
| `cli.new` | `((name (& str)) (about (& str)))` | `cli` |
| `cli.pos` | `((self (&mut cli)) (name (& str)))` | `unit` |
| `cli.opt` | `((self (&mut cli)) (name (& str)) (default (& str)))` | `unit` |
| `cli.flag` | `((self (&mut cli)) (name (& str)))` | `unit` |
| `cli.usage` | `((self (& cli)))` | `str` |
| `cli.parse` | `((self (&mut cli)) (args (vec str)))` | `unit` |
| `cli.die` | `((self (& cli)) (code (& str)) (what (& str)))` | `never` |
| `cli.get` | `((self (& cli)) (name (& str)))` | `(& str)` |
| `cli.get_int` | `((self (& cli)) (name (& str)))` | `i64` |
| `cli.has` | `((self (& cli)) (name (& str)))` | `bool` |
| `cli.at` | `((self (& cli)) (i i64))` | `(& str)` |
| `cli.count` | `((self (& cli)))` | `i64` |

## Tests

`cli_parse`
