const STEPS = 50000;
const DT = 0.01;
const PI = 3.141592653589793;
const DAYS = 365.24;
function nsqrt(x) {
  if (x === 0) return 0;
  let guess = x > 1 ? x * 0.5 : 1;
  for (let i = 0; i < 8; i++) guess = 0.5 * (guess + x / guess);
  return guess;
}
const sm = 4 * PI * PI;
const b = [
  [0, 0, 0, 0, 0, 0, sm],
  [4.8414314424647209, -1.16032004402742839, -0.103622044471123109, 0.00166007664274403694 * DAYS, 0.00769901118419740425 * DAYS, -0.0000690460016972063023 * DAYS, 0.000954791938424326609 * sm],
  [8.34336671824457987, 4.12479856412430479, -0.403523417114321381, -0.00276742510726862411 * DAYS, 0.00499852801234917238 * DAYS, 0.0000230417297573763929 * DAYS, 0.000285885980666130812 * sm],
  [12.894369562139131, -15.1111514016986312, -0.223307578892655734, 0.00296460137564761618 * DAYS, 0.0023784717395948095 * DAYS, -0.0000296589568540237556 * DAYS, 0.0000436624404335156298 * sm],
  [15.3796971148509165, -25.9193146099879641, 0.179258772950371181, 0.00268067772490389322 * DAYS, 0.00162824170038242295 * DAYS, -0.000095159225451971587 * DAYS, 0.0000515138902046611451 * sm],
];
let px = 0, py = 0, pz = 0;
for (const body of b) {
  px += body[3] * body[6];
  py += body[4] * body[6];
  pz += body[5] * body[6];
}
b[0][3] = -px / sm;
b[0][4] = -py / sm;
b[0][5] = -pz / sm;
function advance(dt) {
  for (let i = 0; i < b.length; i++) {
    for (let j = i + 1; j < b.length; j++) {
      const dx = b[i][0] - b[j][0];
      const dy = b[i][1] - b[j][1];
      const dz = b[i][2] - b[j][2];
      const dist = nsqrt(dx * dx + dy * dy + dz * dz);
      const mag = dt / (dist * dist * dist);
      const im = b[i][6];
      const jm = b[j][6];
      b[i][3] -= dx * jm * mag;
      b[i][4] -= dy * jm * mag;
      b[i][5] -= dz * jm * mag;
      b[j][3] += dx * im * mag;
      b[j][4] += dy * im * mag;
      b[j][5] += dz * im * mag;
    }
  }
  for (const body of b) {
    body[0] += dt * body[3];
    body[1] += dt * body[4];
    body[2] += dt * body[5];
  }
}
function energy() {
  let e = 0;
  for (let i = 0; i < b.length; i++) {
    e += 0.5 * b[i][6] * (b[i][3] * b[i][3] + b[i][4] * b[i][4] + b[i][5] * b[i][5]);
    for (let j = i + 1; j < b.length; j++) {
      const dx = b[i][0] - b[j][0];
      const dy = b[i][1] - b[j][1];
      const dz = b[i][2] - b[j][2];
      e -= (b[i][6] * b[j][6]) / nsqrt(dx * dx + dy * dy + dz * dz);
    }
  }
  return e;
}
for (let s = 0; s < STEPS; s++) advance(DT);
console.log(Math.trunc(energy() * 10000));
