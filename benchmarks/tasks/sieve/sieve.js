const N = 5000000;
const mark = new Uint8Array(N + 1);
mark.fill(1);
mark[0] = 0;
mark[1] = 0;
let count = 0;
let last = 0;
for (let i = 2; i <= N; i++) {
  if (mark[i]) {
    count++;
    last = i;
    for (let j = i * i; j <= N; j += i) mark[j] = 0;
  }
}
console.log(count, last);
