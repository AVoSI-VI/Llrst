use std::collections::HashMap;

fn lcg(x: i64) -> i64 {
    (x * 1664525 + 1013904223) & 0xffff_ffff
}

fn main() {
    const N: i64 = 800_000;
    let mut table = HashMap::new();
    let mut x = 1i64;
    for i in 0..N {
        x = lcg(x);
        table.insert(x, i);
    }
    let mut found = 0i64;
    let mut total = 0i64;
    x = 1;
    for _ in 0..N {
        x = lcg(x);
        if let Some(v) = table.get(&x) {
            found += 1;
            total += v;
        }
    }
    println!("{found} {total} {}", table.len());
}
