const N = 500
const ITERS = 10

proc aval(i, j: int): float =
  let ij = i + j
  1.0 / float((ij * (ij + 1)) div 2 + i + 1)

proc mul(v: seq[float], trans: bool): seq[float] =
  result = newSeq[float](N)
  for i in 0..<N:
    var s = 0.0
    for j in 0..<N:
      if trans: s += aval(j, i) * v[j]
      else: s += aval(i, j) * v[j]
    result[i] = s

var u = newSeq[float](N)
for i in 0..<N: u[i] = 1.0
for k in 0..<ITERS:
  let v = mul(u, false)
  u = mul(v, true)
var acc = 0
for x in u:
  acc += int(x * 1000000.0)
echo acc
