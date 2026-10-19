# Canon: tests

## In-file tests

```
(test str_cat
  (let s (str.cat "a" "b"))
  (assert_eq (& s) "ab"))
```

`test` forms live at the end of the file they test. `llr-test FILE_OR_DIR` compiles each
file with `--test`, which makes `main` a runner over all `test` forms.

Asserts: `(assert cond)`, `(assert_eq a b)` for `(& str)`, `i64`, `bool`, `u8`.
Failure prints `F|FILE|LINE|COL|assert|want=..|got=..` and the test is marked failed.

## Golden tests (compiler and tools)

`tests/run/NAME.llr` with `tests/run/NAME.out`: compile, run, compare stdout.
When `NAME.err` exists, stderr must match it. When `NAME.code` exists, it is the exit
status as one integer plus a newline. Without `NAME.code`, the exit status must be 0.
Without `NAME.err`, stderr is not checked.
`tests/err/NAME.llr` with `tests/err/NAME.err`: compile, compare stderr lines exactly.
`tests/panic/NAME.llr` with `tests/panic/NAME.err`: compile, run, stderr must equal `.err`
and the exit code must be 101.
`tests/run.sh` executes all sets. Output: `ok|NAME` or `fail|NAME|want=..|got=..`.
A `.llr` without a sibling `.out`/`.err` is a helper module for a `proj.*` import.
Goldens that point into `std/` shift when std lines move; regenerate with the command in
`HANDOFF.md` section 5 and review the diff.

## Rule

Every new std function gets at least one `test` form in its file. Every new error code gets
one `tests/err` case.
