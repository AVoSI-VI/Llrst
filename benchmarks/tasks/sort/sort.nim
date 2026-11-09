import algorithm

const N = 1000000

proc lcg(x: int): int = (x * 1664525 + 1013904223) and 0xFFFFFFFF

var x = 1
var vals = newSeq[int](N)
for i in 0..<N:
  x = lcg(x)
  vals[i] = x
sort(vals)
var acc = 0
let step = N div 10
var i = 0
while i < N:
  acc = (acc + vals[i] * (i + 1)) and 0x7FFFFFFF
  i += step
echo vals[0], " ", vals[N div 2], " ", vals[N - 1], " ", acc
