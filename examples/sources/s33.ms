// 测试 match
fn main() -> i32 {
    let c = 4;
    let mut b = 5;
    let a = match c {
        5: {
            return 5;
        },
        4: {
            let mut ss = 10;
            while b > 3 {
                ss -= 2;
                b -= 1;
            }
            return ss;
        }
    };
    return a;
}