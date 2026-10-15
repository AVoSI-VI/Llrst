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
`tests/err/NAME.llr` with `tests/err/NAME.err`: compile, compare stderr lines exactly.
`tests/run.sh` executes both sets. Output: `ok|NAME` or `fail|NAME|want=..|got=..`.

## Rule

Every new std function gets at least one `test` form in its file. Every new error code gets
one `tests/err` case.
