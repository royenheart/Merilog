use std::fs::File;

use Merilog::lex::preprocessor::preprocessor;

fn test(name: &str) {
    let mut path = std::env::current_dir().unwrap();
    path.push(name);
    let file = File::open(path).unwrap();
    let s = preprocessor(&file);
    println!("{}", s);
}

fn main() {
    test("examples/sources/s1.ms");
    test("examples/sources/s2.ms");
    test("examples/sources/s3.ms");
    test("examples/sources/s4.ms");
}