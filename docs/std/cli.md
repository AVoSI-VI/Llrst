# std.cli

Argument parsing for tools.

Source: `std/cli.llr`. Import with `(use std.cli)` for module functions; types and methods need no import.

## Types

- `(struct cli (name str) (about str) (pos_names (vec str)) (opt_names (vec str)) (opt_vals (vec str)) (flag_names (vec str)) (flag_set (vec bool)) (pos (vec str)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `cli.new` | `((name (& str)) (about (& str)))` | `cli` |
| `cli.pos` | `((self (&mut cli)) (name (& str)))` | `unit` |
| `cli.opt` | `((self (&mut cli)) (name (& str)) (default (& str)))` | `unit` |
| `cli.flag` | `((self (&mut cli)) (name (& str)))` | `unit` |
| `cli.parse` | `((self (&mut cli)) (args (vec str)))` | `unit` |
| `cli.get` | `((self (& cli)) (name (& str)))` | `(& str)` |
| `cli.get_int` | `((self (& cli)) (name (& str)))` | `i64` |
| `cli.has` | `((self (& cli)) (name (& str)))` | `bool` |
| `cli.at` | `((self (& cli)) (i i64))` | `(& str)` |
| `cli.count` | `((self (& cli)))` | `i64` |

## Tests

`cli_parse`
