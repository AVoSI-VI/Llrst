const N = 800000;
function lcg(x) {
  return (x * 1664525 + 1013904223) % 4294967296;
}
const table = new Map();
let x = 1;
for (let i = 0; i < N; i++) {
  x = lcg(x);
  table.set(x, i);
}
let found = 0;
let total = 0;
x = 1;
for (let i = 0; i < N; i++) {
  x = lcg(x);
  if (table.has(x)) {
    found++;
    total += table.get(x);
  }
}
console.log(found, total, table.size);
