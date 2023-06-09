// 测试函数递归调用
fn fib(n: i32) -> i32 {
    if n < 3 {
        return 1;
    }
    let p1 = n - 1;
    let p2 = n - 2;
    return fib(p1) + fib(p2);
}

fn main() -> i32 {
    return fib(5);
}