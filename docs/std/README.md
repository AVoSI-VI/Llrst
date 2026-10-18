# std index

| module | role |
|--------|------|
| [sys](sys.md) | libc externs and the panic runtime. Not for programs; std only. |
| [str](str.md) | Owned byte string `str`. Literals are `(& str)`. |
| [vec](vec.md) | Growable array `(vec T)`. |
| [opt](opt.md) | Optional `(opt T)`: `(opt.some x)` / `(opt.none)`. |
| [res](res.md) | Result `(res T E)`: `(res.ok x)` / `(res.err e)`. |
| [io](io.md) | Standard streams. |
| [os](os.md) | Process environment. |
| [fs](fs.md) | Files and directories. Errors are `res.err` strings like `io.read|path=x`. |
| [path](path.md) | Path string helpers; `/` only. |
| [map](map.md) | Hash map `(map K V)`. Hash and eq are function pointers. |
| [fmt](fmt.md) | Pipe-record formatting for model-facing output. |
| [json](json.md) | Minimal JSON encoding. |
| [cli](cli.md) | Argument parsing for tools. |
| [sexp](sexp.md) | S-expression reader used by `llr-fmt` and `llr-doc`. |
| [ast](ast.md) | Surface grammar parsed from a sexp tree. `llrc` remains the checker. |
| [glob](glob.md) | Glob matching and directory walking. |
| [re](re.md) | Backtracking regex: `. ^ $ * + ? {n,m} [..] [^..] \d \w \s ( ) |`. |
| [diff](diff.md) | Line diff by LCS, pipe records. |
| [test](test.md) | Test harness used by `llrc build --test` and `llr-test`. |

Rules: one module per file, every pub fn has a test form in its file, `llr-doc` regenerates the signature tables (see docs/workflows/std.md).
