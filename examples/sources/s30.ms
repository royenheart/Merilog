struct aaa {
    a: f32,
    b: i32,
    fn get_a(a: f32) -> f32 {
        let c = self->a;
        return c;
    },
    fn get_a_add_r(r: f32) -> f32 {
        let c = self->a + r;
        return c;
    } 
}

// 测试隐藏机制
fn main() -> f32 {
    let mut a = aaa(3.4, 4);
    let c = "23232";
    _ = a->get_a(45.2);
    let ss = a->get_a_add_r(1.90);
    return ss;
}