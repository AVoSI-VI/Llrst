struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

fn make(depth: i32) -> Node {
    if depth == 0 {
        Node { left: None, right: None }
    } else {
        Node {
            left: Some(Box::new(make(depth - 1))),
            right: Some(Box::new(make(depth - 1))),
        }
    }
}

fn check(node: &Node) -> i64 {
    match &node.left {
        None => 1,
        Some(left) => 1 + check(left) + check(node.right.as_ref().unwrap()),
    }
}

fn main() {
    const MIN_D: i32 = 4;
    const MAX_D: i32 = 13;
    let stretch = check(&make(MAX_D + 1));
    let long_lived = make(MAX_D);
    let mut total = 0i64;
    let mut depth = MIN_D;
    while depth <= MAX_D {
        let iterations = 1i64 << (MAX_D - depth + MIN_D);
        let mut acc = 0i64;
        let mut i = 0i64;
        while i < iterations {
            acc += check(&make(depth));
            i += 1;
        }
        total += acc;
        depth += 2;
    }
    total += check(&long_lived);
    println!("{stretch} {total}");
}
