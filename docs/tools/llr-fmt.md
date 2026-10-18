# llr-fmt

Canonical layout for `.llr` files and the canon linter.

```
llr-fmt FILE... [--width N] [--check] [--stdout] [--lint]
```

| mode | effect |
|------|--------|
| default | rewrites each file in place, prints `ok|FILE|formatted` or `ok|FILE|unchanged` |
| `--check` | no writes; `ok|FILE|formatted` or `diff|FILE|needs format`; exit 2 if any diff |
| `--stdout` | prints the formatted text |
| `--lint` | prints canon warnings `W|FILE|LINE|COL|0|CODE|k=v`; exit 2 if any |

Layout rules: a form is one line when it fits in `N` columns (`--width`, default 100) and
holds no comment. A comment anywhere inside a form forces a line break even when the flat
text would fit. Otherwise the header stays on the first line and each remaining item goes
on its own line indented by 2. Header sizes: `fn` 4 (5 with generics), `struct`/`enum`/`impl` 2, `if`
`while` `match` `set` `test` 2, `for` 4, `let` 2 (3 with `mut`), `do` `loop` 1, calls 2.
Top-level forms are separated by one blank line except consecutive `use` forms and a form
after a comment. Comments are kept verbatim on their own line.

Lint codes: `canon.program.file_lines` (>500), `canon.program.fn_lines` (>40),
`canon.program.use_first`, `canon.program.use_sorted`, `canon.program.main_last`,
`canon.memory.raw` (`ptr.*` or `extern` outside std), `canon.flow.nested_if` (depth > 2),
`canon.bind.unit` (`(let _ ...)`).
