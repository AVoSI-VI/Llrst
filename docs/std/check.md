# std.check

Type and name checker over std.ast. Builds a typed tree. Borrow checking stays in llrc.

Source: `std/check.llr`. Import with `(use std.check)` for module functions; types and methods need no import.

## Types

- `(struct info (pub (show str)) (pub (module str)) (pub (tname str)))`
- `(struct def (pub (name str)) (pub (qname str)) (pub (is_type bool)) (pub (is_method bool)) (pub (is_pub bool)) (pub (module str)) (pub (si i64)) (pub (ii i64)) (pub (fi i64)) (pub (gens (vec str))))`
- `(struct source (pub (module str)) (pub (items (vec item))))`
- `(struct loc (pub (name str)) (pub (ty info)) (pub (mut bool)) (pub (line i64)) (pub (col i64)) (pub (len i64)))`
- `(struct texpr (pub (kind u8)) (pub (ty str)) (pub (callee str)) (pub (builtin i64)) (pub (kids (vec texpr))) (pub (line i64)) (pub (col i64)) (pub (len i64)))`
- `(struct typed (pub (ty info)) (pub (ex texpr)))`
- `(struct prog (pub (srcs (vec source))) (pub (defs (vec def))) (pub (main i64)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `check.dot_at` | `((s (& str)))` | `i64` |
| `check.last_dot` | `((s (& str)))` | `i64` |
| `check.span` | `((line i64) (col i64) (len i64) (code (& str)) (rest (& str)))` | `str` |
| `check.kv` | `((k (& str)) (v (& str)))` | `str` |
| `check.join_kv` | `((a (& str)) (av (& str)) (b (& str)) (bv (& str)))` | `str` |
| `check.find_def` | `((defs (& (vec def))) (module (& str)) (name (& str)) (want_type bool))` | `i64` |
| `check.find_type` | `((defs (& (vec def))) (srcs (& (vec source))) (from i64) (path (& str)))` | `i64` |
| `check.find_fn` | `((defs (& (vec def))) (srcs (& (vec source))) (from i64) (path (& str)))` | `i64` |
| `check.ret_of` | `((srcs (& (vec source))) (d (& def)))` | `(& ty)` |
| `check.priv` | `((line i64) (col i64) (len i64) (name (& str)) (module (& str)))` | `str` |
| `check.nparams` | `((srcs (& (vec source))) (d (& def)))` | `i64` |
| `check.ngens` | `((srcs (& (vec source))) (d (& def)))` | `i64` |
| `check.is_int` | `((s (& str)))` | `bool` |
| `check.is_num` | `((s (& str)))` | `bool` |
| `check.op_name` | `((s (& str)))` | `str` |
| `check.is_cmp` | `((op (& str)))` | `bool` |
| `check.bin_ok` | `((op (& str)) (show (& str)))` | `bool` |
| `check.which_bi` | `((q (& str)) (text (& str)))` | `i64` |
| `check.clone_info` | `((a (& info)))` | `info` |
| `check.mod_si` | `((srcs (& (vec source))) (module (& str)))` | `i64` |
| `check.load_prog` | `((path (& str)) (over (& str)))` | `(res prog str)` |
| `check.check` | `((path (& str)) (over (& str)))` | `(res (vec texpr) (vec str))` |

## Tests

`check_mismatch`, `check_drop`
