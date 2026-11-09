const MIN_D = 4;
const MAX_D = 13;
function make(depth) {
  if (depth === 0) return { left: null, right: null };
  return { left: make(depth - 1), right: make(depth - 1) };
}
function check(node) {
  if (node.left === null) return 1;
  return 1 + check(node.left) + check(node.right);
}
const stretch = check(make(MAX_D + 1));
const longLived = make(MAX_D);
let total = 0;
for (let depth = MIN_D; depth <= MAX_D; depth += 2) {
  const iterations = 1 << (MAX_D - depth + MIN_D);
  let acc = 0;
  for (let i = 0; i < iterations; i++) acc += check(make(depth));
  total += acc;
}
total += check(longLived);
console.log(stretch, total);
