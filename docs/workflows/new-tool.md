# Workflow: add a tool

1. Copy the smallest similar tool: `tools/llr-read.llr` for a file tool, `tools/llr-find.llr`
   for a walker. Name it `tools/llr-NAME.llr`.
2. Declare arguments with `cli.new`, `cli.pos`, `cli.opt`, `cli.flag`, then `cli.parse`.
3. Print records with `fmt.pipe2..pipe5` and `fmt.kv`; escape free text with `fmt.field`.
4. Exit codes: 0 ok, 1 tool error (`io.eputs "E|<NAME>|0|0|0|code|k=v"` then `(os.exit 1)`),
   2 negative result.
5. Build and try: `llrc build tools/llr-NAME.llr -o /tmp/llr-NAME`.
6. `llr-fmt tools/llr-NAME.llr --lint` must be clean.
7. Document in `docs/tools/llr-NAME.md` (flags table, output grammar, exit codes, one canon
   invocation), add a row to `docs/tools/README.md`, and add the name to the `tools/` cell
   in `README.md`.
8. `scripts/install.sh` and `scripts/install.ps1` pick up every `tools/llr-*.llr` automatically.
