import std/json
import std/strutils

const N = 800000
var parts = newSeq[string](N)
for i in 0..<N:
  parts[i] = "{\"id\":" & $i & ",\"ok\":true}"
let text = "[" & parts.join(",") & "]"
let values = parseJson(text)
var total = 0
for obj in values:
  total += obj["id"].getInt
echo total, " ", values.len
