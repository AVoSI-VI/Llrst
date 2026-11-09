const W = 600
const LIM = 50
var count = 0
var y = 0
while y < W:
  var x = 0
  while x < W:
    let cr = 2.0 * float(x) / float(W) - 1.5
    let ci = 2.0 * float(y) / float(W) - 1.0
    var zr = 0.0
    var zi = 0.0
    var inside = true
    var k = 0
    while k < LIM:
      let zr2 = zr * zr - zi * zi + cr
      zi = 2.0 * zr * zi + ci
      zr = zr2
      if zr * zr + zi * zi > 4.0:
        inside = false
        break
      k += 1
    if inside: count += 1
    x += 1
  y += 1
echo count
