# Workflow: edit

Goal: one change, verified.

1. `llr-read FILE --from A --to B` to get exact text with line numbers.
2. Compose OLD from the lines verbatim (indentation included). Make OLD unique: include
   the enclosing `(fn name` line when in doubt.
3. `llr-edit FILE OLD NEW`
   - `ok|FILE|replaced=1`: continue.
   - `edit.ambiguous|count=N`: extend OLD; use `--all` only for mechanical renames.
   - `edit.not_found`: re-read; whitespace drifted.
   For multi-line text write OLD and NEW to temp files and pass `--files`.
4. `llr-fmt FILE` to normalize layout (the compiler does not care; the next reader does).
5. `llrc check FILE` and read `E|` lines; see [debug.md](debug.md).
6. `llr-test FILE` (or the directory) and read `ok|` / `fail|` lines.
7. `llr-fmt FILE --lint` must print nothing.
