fn main() -> i32 {
    let mut a = 8;

    if a > 5 {
        let b = 5;
        a = 12;
    } else {
        let c = 6;
        if c < 5 {
            return 7;
        } else {
            return 8;
        }
    }

    return a;
} 