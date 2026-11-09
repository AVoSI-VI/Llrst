const STEPS = 50000
const DT = 0.01
const PI = 3.141592653589793
const DAYS = 365.24

proc nsqrt(x: float): float =
  if x == 0.0: return 0.0
  result = if x > 1.0: x * 0.5 else: 1.0
  for i in 0..<8:
    result = 0.5 * (result + x / result)

let sm = 4.0 * PI * PI
var b = @[
  @[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, sm],
  @[4.84143144246472090, -1.16032004402742839, -0.103622044471123109,
    0.00166007664274403694 * DAYS, 0.00769901118419740425 * DAYS,
    -0.0000690460016972063023 * DAYS, 0.000954791938424326609 * sm],
  @[8.34336671824457987, 4.12479856412430479, -0.403523417114321381,
    -0.00276742510726862411 * DAYS, 0.00499852801234917238 * DAYS,
    0.0000230417297573763929 * DAYS, 0.000285885980666130812 * sm],
  @[12.8943695621391310, -15.1111514016986312, -0.223307578892655734,
    0.00296460137564761618 * DAYS, 0.00237847173959480950 * DAYS,
    -0.0000296589568540237556 * DAYS, 0.0000436624404335156298 * sm],
  @[15.3796971148509165, -25.9193146099879641, 0.179258772950371181,
    0.00268067772490389322 * DAYS, 0.00162824170038242295 * DAYS,
    -0.0000951592254519715870 * DAYS, 0.0000515138902046611451 * sm],
]

var px = 0.0
var py = 0.0
var pz = 0.0
for body in b:
  px += body[3] * body[6]
  py += body[4] * body[6]
  pz += body[5] * body[6]
b[0][3] = -px / sm
b[0][4] = -py / sm
b[0][5] = -pz / sm

proc advance(dt: float) =
  for i in 0..<b.len:
    for j in (i + 1)..<b.len:
      let dx = b[i][0] - b[j][0]
      let dy = b[i][1] - b[j][1]
      let dz = b[i][2] - b[j][2]
      let dist = nsqrt(dx * dx + dy * dy + dz * dz)
      let mag = dt / (dist * dist * dist)
      let im = b[i][6]
      let jm = b[j][6]
      b[i][3] -= dx * jm * mag
      b[i][4] -= dy * jm * mag
      b[i][5] -= dz * jm * mag
      b[j][3] += dx * im * mag
      b[j][4] += dy * im * mag
      b[j][5] += dz * im * mag
  for body in b.mitems:
    body[0] += dt * body[3]
    body[1] += dt * body[4]
    body[2] += dt * body[5]

proc energy(): float =
  for i in 0..<b.len:
    result += 0.5 * b[i][6] * (b[i][3] * b[i][3] + b[i][4] * b[i][4] + b[i][5] * b[i][5])
    for j in (i + 1)..<b.len:
      let dx = b[i][0] - b[j][0]
      let dy = b[i][1] - b[j][1]
      let dz = b[i][2] - b[j][2]
      result -= b[i][6] * b[j][6] / nsqrt(dx * dx + dy * dy + dz * dz)

for s in 0..<STEPS:
  advance(DT)
echo int(energy() * 10000.0)
