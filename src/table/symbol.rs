//! 符号表

use crate::lex::Tokens;

struct table {
    symbols: Vec<Tokens>,
}

impl table {
    fn new() -> Self {
        table { symbols: Vec }
    }
}