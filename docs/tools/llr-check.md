# llr-check

Type and name check. Borrow errors stay in `llrc`.

```
llr-check FILE
```

Prints the first error in the same `E|FILE|LINE|COL|LEN|CODE|k=v` form as `llrc check`, then exits 1. No error exits 0. The file path in the record is the path given on the command line.

It loads `$LLRST_STD` (or `std/`) and any `proj.NAME` module beside FILE. A clean file builds a typed tree (`texpr`: type, callee, builtin id) and exits 0. The first error is printed. It reports `type.mismatch`, `type.arity`, `type.not_callable`, `type.no_field`, `type.no_variant`, `type.infer`, `type.not_numeric`, `type.not_bool`, `type.generic_arity`, `name.unknown`, `name.private`, `name.private_field`, `name.private_variant`, `mut.required`, `clos.nested`, `inline.reject`, `trait.dup`, `trait.missing`, and `trait.object`. `borrow.*`, `move.*`, and `init.*` stay in `llrc`.

Exit: 0 clean, 1 a type or name error, or a read failure (`E|<check>|0|0|0|...` is not used; load failures are returned as the diagnostic body).

Canon: `llr-check tests/err/type_mismatch.llr`.
