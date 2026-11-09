fn aval(i: i64, j: i64) -> f64 {
    let ij = i + j;
    1.0 / ((ij * (ij + 1)) / 2 + i + 1) as f64
}

fn mul(v: &[f64], trans: bool) -> Vec<f64> {
    let n = v.len() as i64;
    let mut out = vec![0.0; v.len()];
    let mut i = 0i64;
    while i < n {
        let mut s = 0.0;
        let mut j = 0i64;
        while j < n {
            let a = if trans { aval(j, i) } else { aval(i, j) };
            s += a * v[j as usize];
            j += 1;
        }
        out[i as usize] = s;
        i += 1;
    }
    out
}

fn main() {
    const N: usize = 500;
    const ITERS: i64 = 10;
    let mut u = vec![1.0; N];
    let mut k = 0i64;
    while k < ITERS {
        let v = mul(&u, false);
        u = mul(&v, true);
        k += 1;
    }
    let mut acc = 0i64;
    for x in u {
        acc += (x * 1_000_000.0) as i64;
    }
    println!("{acc}");
}
