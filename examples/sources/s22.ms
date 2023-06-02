fn main() -> i32 {
    let mut a = 8;

    {
        a += 1;
    } loop a <= 13;

    return a;
} 