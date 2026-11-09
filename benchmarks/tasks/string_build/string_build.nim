import std/strutils

const N = 800000
var parts = newSeq[string](N)
for i in 0..<N:
  parts[i] = $i
let text = parts.join(",")
var acc = 0
for ch in text:
  acc = (acc * 131 + ord(ch)) and 0xFFFFFFFF
echo text.len, " ", acc
