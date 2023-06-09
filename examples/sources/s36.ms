// 测试：分支、循环语句中判定条件不是可作为布尔型判定的条件
fn main() -> i32 {
    let a = 4.0;
    if a {
        return 4;
    }

    return 10;
}