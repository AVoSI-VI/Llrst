const N = 9

proc flips(perm: seq[int]): int =
  var q = perm
  while q[0] != 0:
    let k = q[0]
    var i = 0
    var j = k
    while i < j:
      let t = q[i]
      q[i] = q[j]
      q[j] = t
      i += 1
      j -= 1
    result += 1

var perm = newSeq[int](N)
for i in 0..<N: perm[i] = i
var best = 0
var checksum = 0
var sign = 1

proc rec(k: int) =
  if k == 1:
    if perm[0] != 0:
      let got = flips(perm)
      checksum += sign * got
      if got > best: best = got
    sign = -sign
    return
  rec(k - 1)
  var i = 0
  while i < k - 1:
    let t = perm[0]
    var j = 0
    while j < k - 1:
      perm[j] = perm[j + 1]
      j += 1
    perm[k - 1] = t
    rec(k - 1)
    i += 1

rec(N)
echo checksum, " ", best
