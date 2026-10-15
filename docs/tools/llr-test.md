# llr-test

Builds every file containing `(test ...)` forms with `llrc build --test` and runs it.

```
llr-test [PATH] [--filter NAME]
```

PATH is a file or a directory (default `.`); directories are searched for `*.llr`.
`--filter` runs only tests whose name contains NAME. The `llrc` used is `$LLRC` when set
(may include flags, e.g. `LLRC="llrc --std ./std"`), otherwise `llrc` on PATH.

Output grammar:

```
file|PATH
ok|TEST
fail|TEST|code=N          ; N = 101 assert or panic
fail|TEST|signal=S
F|FILE|LINE|COL|assert_eq|want=..|got=..     ; stderr, from the failing assert
done|fails=N              ; per file
done|files=F|failed_files=N
```

Each test runs in a forked child, so a crash in one test does not stop the others.
Exit: 0 all passed, 2 any failure (including compile errors, printed as `E|...`).
