# std.mqtt

An MQTT 3.1.1 broker. Remaining length is the 1-to-4-byte integer. Subscribe filters may use `+` and `#`. Publish is QoS 0, 1, or 2. Username and password are checked when `serve` is given a user. A will is published when the client drops without DISCONNECT. `(mqtt.serve fd user pass stop_after)` accepts connections until that many publishes have been delivered. `stop_after` 0 runs until the listen socket fails.

CONNECT is `0x10`. The protocol is `MQTT` at level 4. Flags are clean session only (`0x02`). The client id is a non-empty string. Anything else is `mqtt.connect`. The answer is CONNACK `20 02 00 00`. CONNECT must be the first packet.

SUBSCRIBE is `0x82`. One topic filter, a packet id, and QoS 0. `+` or `#` is `mqtt.topic`. Any other QoS is `mqtt.qos`. The answer is SUBACK `0x90`, the same packet id, and return code 0. A second SUBSCRIBE replaces the topic.

PUBLISH is `0x30`. QoS 0, no retain, and no dup. If the topic equals the subscription, that same PUBLISH is written back. A publish before subscribe, or to another topic, is not echoed. Any other type is `mqtt.type|ty=`.

No username, password, will, keepalive handling, or wildcards. `(mqtt.serve fd)` accepts one connection and returns `1` after it has echoed one matching publish.

Source: `std/mqtt.llr`. Import with `(use std.mqtt)` for module functions; types and methods need no import.

## Functions

| name | params | returns |
|------|--------|---------|
| `mqtt.serve` | `((fd i64) (user (& str)) (pass (& str)) (stop_after i64))` | `(res i64 str)` |

## Tests

`mqtt_packets`, `mqtt_http`
