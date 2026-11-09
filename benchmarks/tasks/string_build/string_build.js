const N = 800000;
const parts = [];
for (let i = 0; i < N; i++) parts.push(String(i));
const text = parts.join(",");
let acc = 0;
for (let k = 0; k < text.length; k++) {
  acc = (acc * 131 + text.charCodeAt(k)) >>> 0;
}
console.log(text.length, acc);
