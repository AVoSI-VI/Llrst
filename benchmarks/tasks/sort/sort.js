const N = 1000000;
function lcg(x) {
  return (x * 1664525 + 1013904223) % 4294967296;
}
let x = 1;
const vals = [];
for (let i = 0; i < N; i++) {
  x = lcg(x);
  vals.push(x);
}
vals.sort((a, b) => a - b);
let acc = 0;
const step = Math.floor(N / 10);
for (let i = 0; i < N; i += step) {
  acc = (acc + vals[i] * (i + 1)) % 2147483648;
}
console.log(vals[0], vals[Math.floor(N / 2)], vals[N - 1], acc);
