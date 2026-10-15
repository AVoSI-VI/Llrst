# Workflow: grow std

1. One module per file: `std/NAME.llr`. Types in std are visible everywhere; module
   functions need `(use std.NAME)`.
2. Methods go in `(impl TYPE ...)` and are called `TYPE.method`. Free functions are `pub`.
3. Every new function gets a `(test ...)` form at the end of the file. Run with
   `llr-test std/NAME.llr` or `llrc build std/NAME.llr --test -o /tmp/t && /tmp/t`.
4. Raw memory (`ptr.*`, `extern`) is allowed only in std; wrap it so programs never see it.
5. Keep files under 500 lines and functions under 40 lines (`llr-fmt --lint`).
6. Update `docs/std/NAME.md`: add the signature row and a canon line in `docs/canon/`.
7. Run `tests/run.sh` before committing: it rebuilds every std test and the golden tests.
