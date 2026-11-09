fn main() {
    const N: i64 = 800_000;
    let mut text = String::new();
    for i in 0..N {
        if i > 0 {
            text.push(',');
        }
        text.push_str(&i.to_string());
    }
    let mut acc = 0i64;
    for byte in text.bytes() {
        acc = (acc * 131 + byte as i64) & 0xffff_ffff;
    }
    println!("{} {acc}", text.len());
}
