const W = 600;
const LIM = 50;
let count = 0;
for (let y = 0; y < W; y++) {
  for (let x = 0; x < W; x++) {
    const cr = (2.0 * x) / W - 1.5;
    const ci = (2.0 * y) / W - 1.0;
    let zr = 0;
    let zi = 0;
    let inside = true;
    for (let k = 0; k < LIM; k++) {
      const zr2 = zr * zr - zi * zi + cr;
      zi = 2 * zr * zi + ci;
      zr = zr2;
      if (zr * zr + zi * zi > 4) {
        inside = false;
        break;
      }
    }
    if (inside) count++;
  }
}
console.log(count);
