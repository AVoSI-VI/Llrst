# Agent tools

Tools an agent uses most while programming, ranked by call frequency in typical coding
sessions, and the Llrst reimplementation of each. All tools print one record per line,
`|` separated, first field is a tag. Diagnostics go to stderr as `E|...`. Exit codes:
0 ok, 1 tool error, 2 negative result (no matches, differences, failed tests).

| rank | agent tool | role | Llrst tool |
|------|-----------|------|-----------|
| 1 | read file | view source with line numbers | [llr-read](llr-read.md) |
| 2 | grep / ripgrep | find text or regex across files | [llr-grep](llr-grep.md) |
| 3 | edit / string replace | apply an exact replacement | [llr-edit](llr-edit.md) |
| 4 | shell: build and run | compile, run, see errors | `llrc build`, `llrc run`, `llrc check` |
| 5 | glob / find | locate files by name pattern | [llr-find](llr-find.md) |
| 6 | test runner | run tests, read failures | [llr-test](llr-test.md) |
| 7 | formatter / linter | normalize layout, enforce canon | [llr-fmt](llr-fmt.md) |
| 8 | diff | compare two files | [llr-diff](llr-diff.md) |
| | write file | create or replace a whole file from stdin, creating parents | [llr-write](llr-write.md) |
| | list directory | sorted recursive listing | [llr-ls](llr-ls.md) |
| | remove file | delete a file or a directory tree | [llr-rm](llr-rm.md) |
| | git read | status, diff, and log; no commit | [llr-git](llr-git.md) |
| | std docs | regenerate signature tables | [llr-doc](llr-doc.md) |
| | type check | `type.*` and `name.*` errors | [llr-check](llr-check.md) |
| | HTTP server | files and exact routes on one poll loop | [llr-http](llr-http.md) |

Every tool is a single `.llr` file under `tools/` built by `scripts/install.sh`. Shared
behaviour lives in `std` (`cli`, `fmt`, `fs`, `glob`, `re`, `diff`, `sexp`), so a tool is
mostly argument parsing plus output.

Common flags: `--help` prints `usage|NAME ARGS...|about`. Every tool with a list output
accepts `--max N`. A truncated list ends with `more|N`, where `N` is the number of records
not shown. `llr-diff` and `llr-test` default `--max` to 0, which means no limit. `llr-git log`
passes N to `git log -n` and does not print `more|`.
