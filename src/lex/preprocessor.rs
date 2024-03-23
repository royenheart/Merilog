//! 预处理 - 多个空格替换成一个空格，去除没必要的空行

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub fn preprocessor(file: &File) -> String {
    let r = BufReader::new(file);
    let mut raw = String::with_capacity(r.capacity());
    let cursor = r.lines().map(|x| x.unwrap());
    for line in cursor {
        let line = line.trim();
        if line.is_empty() || (line.starts_with("//") && !line.starts_with("//!")) {
            continue;
        }
        let parse = line.split(' ');
        for word in parse {
            if word.is_empty() {
                continue;
            }
            raw.push_str(word.trim());
            raw.push(' ');
        }
        raw.push('\n');
    }
    raw
}
