# std.test

Test harness used by `llrc build --test` and `llr-test`.

Source: `std/test.llr`. Import with `(use std.test)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `test.fail` | `((loc (& str)) (msg (& str)))` | `never` |
| `test.eq_str` | `((loc (& str)) (got (& str)) (want (& str)))` | `unit` |
| `test.eq_int` | `((loc (& str)) (got i64) (want i64))` | `unit` |
| `test.eq_bool` | `((loc (& str)) (got bool) (want bool))` | `unit` |
| `test.eq_f64` | `((loc (& str)) (got f64) (want f64) (eps f64))` | `unit` |
| `test.ne_int` | `((loc (& str)) (got i64) (want i64))` | `unit` |
| `test.ne_str` | `((loc (& str)) (got (& str)) (want (& str)))` | `unit` |
| `test.eq_by <T:Eq:Show>` | `((loc (& str)) (got (& T)) (want (& T)))` | `unit` |
| `test.ne_by <T:Eq:Show>` | `((loc (& str)) (got (& T)) (want (& T)))` | `unit` |
| `test.run` | `((name (& str)) (f (fn () unit)))` | `i64` |
| `test.done` | `((fails i64))` | `unit` |

## Tests

`harness_eq`
