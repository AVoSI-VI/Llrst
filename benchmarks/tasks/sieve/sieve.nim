const N = 5000000
var mark = newSeq[byte](N + 1)
for i in 0..N: mark[i] = 1
mark[0] = 0
mark[1] = 0
var count = 0
var last = 0
var i = 2
while i <= N:
  if mark[i] != 0:
    count += 1
    last = i
    var j = i * i
    while j <= N:
      mark[j] = 0
      j += i
  i += 1
echo count, " ", last
