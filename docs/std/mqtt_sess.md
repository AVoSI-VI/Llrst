# std.mqtt_sess

MQTT topic filters. + is one level. # is the rest and only at the end.

Source: `std/mqtt_sess.llr`. Import with `(use std.mqtt_sess)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `mqtt_sess.fits` | `((filter (& str)) (topic (& str)))` | `bool` |
| `mqtt_sess.keep` | `((ticks i64) (k i64))` | `bool` |

## Tests

`mqtt_match`
