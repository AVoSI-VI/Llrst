# llr-test

Builds every file containing `(test ...)` forms with `llrc build --test` and runs it.

```
llr-test [PATH] [--filter NAME] [--max N]
```

PATH is a file or a directory (default `.`); directories are searched for `*.llr`.
`--filter` runs only tests whose name contains NAME. `--max N` (default 0, no limit)
runs at most N files. `more|N` is how many files were not run. `done|files=` still
counts every file found, and `failed_files` counts only files that ran. The `llrc` used
is `$LLRC` when set (may include flags, e.g. `LLRC="llrc --std ./std"`), otherwise `llrc`
on PATH.

Output grammar:

```
file|PATH
ok|TEST
fail|TEST|code=N          ; N = 101 assert or panic
fail|TEST|signal=S
F|FILE|LINE|COL|assert_eq|want=..|got=..     ; stderr, from the failing assert
done|fails=N              ; per file
more|N                    ; files not run because of --max
done|files=F|failed_files=N
```

On Unix each test runs in a forked child. On Windows (`sys.os_id` 2) the harness re-execs
the test binary with `LLR_TEST_CHILD` set to that test's name, because a Cranelift or
`self/` stack cannot be forked. Either way a crash in one test does not stop the others.
Exit: 0 all passed, 2 any failure among the files that ran (including compile errors,
printed as `E|...`). Truncation alone is not exit 2.
