# llr-doc

Regenerate a `docs/std` page from the public signatures in a `.llr` file.

```
llr-doc FILE.llr... [--out DIR]
```

| flag | default | meaning |
|------|---------|---------|
| `FILE.llr` | | one or more modules |
| `--out` | stdout | write `DIR/NAME.md` for each file |

Private items are skipped. A public field is printed as `(pub (name type))`. A method is `TYPE.name`; a free function is `module.name`. The module name is the file stem.

When `DIR/NAME.md` already exists, the text before the first `## ` is kept. A new page uses the file's first comment as the summary.

Exit: 0 written, 1 read or parse error (`E|<doc>|0|0|0|...`).

Canon: `llr-doc --out docs/std std/vec.llr`.
