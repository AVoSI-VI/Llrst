# std.mqtt_pkt

MQTT 3.1.1 remaining length and strings.

Source: `std/mqtt_pkt.llr`. Import with `(use std.mqtt_pkt)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `mqtt_pkt.put_str` | `((out (&mut str)) (s (& str)))` | `unit` |
| `mqtt_pkt.put_rem` | `((out (&mut str)) (n i64))` | `unit` |
| `mqtt_pkt.read_rem` | `((buf (& str)) (i (&mut i64)))` | `(res i64 str)` |
| `mqtt_pkt.u16` | `((buf (& str)) (i i64))` | `i64` |
| `mqtt_pkt.read_str` | `((buf (& str)) (i (&mut i64)) (end i64))` | `(res str str)` |

## Tests

`mqtt_rem`
