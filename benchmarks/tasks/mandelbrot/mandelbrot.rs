fn main() {
    const W: i64 = 600;
    const LIM: i64 = 50;
    let mut count = 0i64;
    let mut y = 0i64;
    while y < W {
        let mut x = 0i64;
        while x < W {
            let cr = 2.0 * x as f64 / W as f64 - 1.5;
            let ci = 2.0 * y as f64 / W as f64 - 1.0;
            let mut zr = 0.0;
            let mut zi = 0.0;
            let mut inside = true;
            let mut k = 0i64;
            while k < LIM {
                let zr2 = zr * zr - zi * zi + cr;
                zi = 2.0 * zr * zi + ci;
                zr = zr2;
                if zr * zr + zi * zi > 4.0 {
                    inside = false;
                    break;
                }
                k += 1;
            }
            if inside {
                count += 1;
            }
            x += 1;
        }
        y += 1;
    }
    println!("{count}");
}
