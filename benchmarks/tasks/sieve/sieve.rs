fn main() {
    const N: usize = 5_000_000;
    let mut mark = vec![1u8; N + 1];
    mark[0] = 0;
    mark[1] = 0;
    let mut count: i64 = 0;
    let mut last: i64 = 0;
    let mut i = 2usize;
    while i <= N {
        if mark[i] != 0 {
            count += 1;
            last = i as i64;
            let mut j = i * i;
            while j <= N {
                mark[j] = 0;
                j += i;
            }
        }
        i += 1;
    }
    println!("{count} {last}");
}
