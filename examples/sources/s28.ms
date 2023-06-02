struct aaa {
    a: f32,
    b: i32
}

fn main() -> f32 {
    let a = aaa(3.4, 4);
    let mut r = a->a + 4.354; 
    // r = 7.754
    return r;
}