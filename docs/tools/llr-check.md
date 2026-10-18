# llr-check

Type and name check. Borrow errors stay in `llrc`.

```
llr-check FILE
```

Prints the first error in the same `E|FILE|LINE|COL|LEN|CODE|k=v` form as `llrc check`, then exits 1. No error exits 0. The file path in the record is the path given on the command line.

It loads `$LLRST_STD` (or `std/`) and any `proj.NAME` module beside FILE. It reports `type.mismatch`, `name.private`, and `name.private_field`. Other `type.*` codes and `name.unknown` are not matched to `llrc` yet.

Exit: 0 clean, 1 a type or name error, or a read failure (`E|<check>|0|0|0|...` is not used; load failures are returned as the diagnostic body).

Canon: `llr-check tests/err/type_mismatch.llr`.
