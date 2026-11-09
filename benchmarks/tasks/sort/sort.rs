fn lcg(x: i64) -> i64 {
    (x * 1664525 + 1013904223) & 0xffff_ffff
}

fn main() {
    const N: usize = 1_000_000;
    let mut x = 1i64;
    let mut vals = Vec::with_capacity(N);
    for _ in 0..N {
        x = lcg(x);
        vals.push(x);
    }
    vals.sort();
    let mut acc = 0i64;
    let step = N / 10;
    let mut i = 0usize;
    while i < N {
        acc = (acc + vals[i] * (i as i64 + 1)) & 0x7fff_ffff;
        i += step;
    }
    println!("{} {} {} {acc}", vals[0], vals[N / 2], vals[N - 1]);
}
