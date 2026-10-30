# Workflow: explore

Goal: understand where something lives before changing it.

1. `llr-find "*.llr" DIR --max 50`
   Read `path|PATH|MTIME`; newest files are usually the active area.
2. `llr-grep "\\(fn NAME" DIR` to find a definition. One `hit|PATH|LINE|COL|text` line is the
   definition; several lines mean overloads per type (`(impl T ...)`).
3. `llr-grep "mod\\.NAME" DIR` or `llr-grep "NAME" --fixed DIR --max 50` to list uses.
4. `llr-read PATH --from L-10 --to L+40` around the hit. Prefer windows under 60 lines.
   The next `--from` is one past the last `line|N`. `more|N` is how many lines remain.
5. For a module API, read `docs/std/<mod>.md` instead of the source.
6. Stop when the definition, its callers, and the test file for it are known.
