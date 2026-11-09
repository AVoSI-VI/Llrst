import tables

const N = 800000

proc lcg(x: int): int = (x * 1664525 + 1013904223) and 0xFFFFFFFF

var table = initTable[int, int]()
var x = 1
for i in 0..<N:
  x = lcg(x)
  table[x] = i
var found = 0
var total = 0
x = 1
for i in 0..<N:
  x = lcg(x)
  if table.hasKey(x):
    found += 1
    total += table[x]
echo found, " ", total, " ", table.len
