struct aaa {
    a: f32,
    b: i32
}

// 测试隐藏机制
fn main() -> i32 {
    let a = aaa(3.4, 4);
    let mut a = 4;
    let b = 5.3;
    while a < 12 {
        let b = 3;
        a += b;
    }
    // a = 13
    return a;
}