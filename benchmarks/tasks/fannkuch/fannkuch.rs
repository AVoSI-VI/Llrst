fn flips(perm: &[i64]) -> i64 {
    let mut q = perm.to_vec();
    let mut count = 0i64;
    while q[0] != 0 {
        let k = q[0] as usize;
        let mut i = 0usize;
        let mut j = k;
        while i < j {
            q.swap(i, j);
            i += 1;
            j -= 1;
        }
        count += 1;
    }
    count
}

fn rec(perm: &mut [i64], k: usize, best: &mut i64, checksum: &mut i64, sign: &mut i64) {
    if k == 1 {
        if perm[0] != 0 {
            let got = flips(perm);
            *checksum += *sign * got;
            if got > *best {
                *best = got;
            }
        }
        *sign = -*sign;
        return;
    }
    rec(perm, k - 1, best, checksum, sign);
    let mut i = 0usize;
    while i < k - 1 {
        let t = perm[0];
        let mut j = 0usize;
        while j < k - 1 {
            perm[j] = perm[j + 1];
            j += 1;
        }
        perm[k - 1] = t;
        rec(perm, k - 1, best, checksum, sign);
        i += 1;
    }
}

fn main() {
    const N: usize = 9;
    let mut perm: Vec<i64> = (0..N as i64).collect();
    let mut best = 0i64;
    let mut checksum = 0i64;
    let mut sign = 1i64;
    rec(&mut perm, N, &mut best, &mut checksum, &mut sign);
    println!("{checksum} {best}");
}
