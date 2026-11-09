const N = 500;
const ITERS = 10;
function aval(i, j) {
  const ij = i + j;
  return 1.0 / (Math.floor((ij * (ij + 1)) / 2) + i + 1);
}
function mul(v, trans) {
  const out = new Array(N);
  for (let i = 0; i < N; i++) {
    let s = 0;
    for (let j = 0; j < N; j++) s += (trans ? aval(j, i) : aval(i, j)) * v[j];
    out[i] = s;
  }
  return out;
}
let u = new Array(N).fill(1);
for (let k = 0; k < ITERS; k++) {
  const v = mul(u, false);
  u = mul(v, true);
}
let acc = 0;
for (const x of u) acc += Math.trunc(x * 1000000);
console.log(acc);
