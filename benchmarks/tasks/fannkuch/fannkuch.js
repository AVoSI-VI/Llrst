const N = 9;
function flips(perm) {
  const q = perm.slice();
  let count = 0;
  while (q[0] !== 0) {
    const k = q[0];
    let i = 0;
    let j = k;
    while (i < j) {
      const t = q[i];
      q[i] = q[j];
      q[j] = t;
      i++;
      j--;
    }
    count++;
  }
  return count;
}
const perm = [];
for (let i = 0; i < N; i++) perm.push(i);
let best = 0;
let checksum = 0;
let sign = 1;
function rec(k) {
  if (k === 1) {
    if (perm[0] !== 0) {
      const got = flips(perm);
      checksum += sign * got;
      if (got > best) best = got;
    }
    sign = -sign;
    return;
  }
  rec(k - 1);
  for (let i = 0; i < k - 1; i++) {
    const t = perm[0];
    for (let j = 0; j < k - 1; j++) perm[j] = perm[j + 1];
    perm[k - 1] = t;
    rec(k - 1);
  }
}
rec(N);
console.log(checksum, best);
