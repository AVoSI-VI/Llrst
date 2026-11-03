# llr-check

Runs `llrc check` on a file. The two cannot disagree: `llr-check` is a front end, not a second checker.

```
llr-check FILE [--std DIR] [--ui DIR] [--deny CODE]
```

Every argument is passed to `llrc check` as given, and the child's stderr and exit code are the tool's. It reports whatever `llrc check` reports: `type.*`, `name.*`, `borrow.*`, `move.*`, `init.*`, `match.*`, `trait.*`, `flow.*`, warnings, and `--deny` promotions, in the same `E|FILE|LINE|COL|LEN|CODE|k=v` records.

The compiler is `$LLRC` when set (used as given, even if missing), otherwise `llrc` (`llrc.exe` on Windows) in the directory of the `llr-check` executable, otherwise `llrc` on `PATH`. When none runs: `E|<check>|0|0|0|check.llrc|fix=install llrc or set LLRC`, exit 1.

Exit: 0 clean, 1 any error or a missing compiler.

Canon: `llr-check tests/err/type_mismatch.llr`.
