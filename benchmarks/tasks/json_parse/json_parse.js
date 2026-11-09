const N = 800000;
const parts = [];
for (let i = 0; i < N; i++) parts.push('{"id":' + i + ',"ok":true}');
const text = "[" + parts.join(",") + "]";
const values = JSON.parse(text);
let total = 0;
for (const obj of values) total += obj.id;
console.log(total, values.length);
