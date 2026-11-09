fn main() {
    const N: i64 = 800_000;
    let mut text = String::from("[");
    for i in 0..N {
        if i > 0 {
            text.push(',');
        }
        text.push_str(&format!("{{\"id\":{i},\"ok\":true}}"));
    }
    text.push(']');
    let bytes = text.as_bytes();
    let mut i = 0usize;
    let mut sum = 0i64;
    let mut nums = 0i64;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'-' || (c >= b'0' && c <= b'9') {
            let mut v = 0i64;
            if c != b'-' {
                while i < bytes.len() && bytes[i] >= b'0' && bytes[i] <= b'9' {
                    v = v * 10 + (bytes[i] - b'0') as i64;
                    i += 1;
                }
                sum += v;
                nums += 1;
            }
            continue;
        }
        i += 1;
    }
    println!("{sum} {nums}");
}
