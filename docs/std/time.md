# std.time

Wall clock, monotonic clock, sleep, and UTC calendar dates.

Source: `std/time.llr`. Import with `(use std.time)` for module functions; types and methods need no import.

## Types

- `(struct date (pub (year i64)) (pub (month i64)) (pub (day i64)) (pub (hour i64)) (pub (min i64)) (pub (sec i64)) (pub (ms i64)))`

## Functions

| name | params | returns |
|------|--------|---------|
| `time.now_ms` | `()` | `i64` |
| `time.mono_ms` | `()` | `i64` |
| `time.sleep_ms` | `((ms i64))` | `unit` |
| `time.utc` | `((ms i64))` | `date` |
| `time.iso8601` | `((d (& date)))` | `str` |
| `time.parse_iso` | `((s (& str)))` | `(opt date)` |

## Tests

`time_epoch`, `time_clock`
