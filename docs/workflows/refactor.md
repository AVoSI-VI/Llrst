# Workflow: refactor

Order: reuse, refactor, then new code.

## Rename

1. `llr-grep "OLD" --fixed DIR --max 500` to list every use (word boundaries: use regex
   `"\\bOLD\\b"` is not supported; check each hit instead).
2. Per file: `llr-edit FILE OLD NEW --all`.
3. `llrc check` each changed file, then `llr-test DIR`.

## Extract a function

1. `llr-read` the long function; pick a contiguous run of forms.
2. `llr-edit` the run into a call `(helper args)`.
3. `llr-edit` to insert `(fn helper ARGS RET ...)` above the caller (OLD = the `(fn caller`
   header line; NEW = helper + blank line + the same header).
4. `llr-fmt FILE`; `llrc check FILE`; borrow errors here usually mean a parameter should be
   `(& T)` or `(&mut T)` rather than `T`.

## Move to a new module

1. Create `name.llr` beside `main.llr`; move cohesive `fn`s, mark exported ones `pub`.
2. Add `(use proj.name)` in callers and prefix calls with `name.`.
3. `llr-fmt --lint` enforces `use` ordering; `llr-test DIR`.

## Grow std instead of duplicating

If a helper is generic (strings, vectors, paths, records), add it to the matching `std/`
module with a `test` form and a row in `docs/std/<mod>.md`, then call it from the tool.
