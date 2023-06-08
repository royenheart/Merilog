use crate::mistakes::show::Mis;
use crate::mistakes::show::Types;
use crate::mistakes::show::{Froms, LineType};
use core::panic;
use std::str::Lines;
use std::{char, collections::VecDeque, str::Chars};

use super::Tokens;

const EOF: char = '\0';
const EOF_STR: &str = "\0";

/// 词法分析主体
pub struct Analysis<'a> {
    /// 预处理后的源代码行迭代器
    iter_line: Lines<'a>,
    /// 预处理后的源代码字符迭代器
    iter_c: Chars<'a>,
    /// 当前扫描行内容
    line: &'a str,
    /// 当前扫描行号
    line_offset: usize,
    /// 当前扫描行字符偏移量
    c_offset: usize,
    /// 扫描缓冲区
    buf: VecDeque<Tokens>,
    peek: char,
    file: &'a str,
}

impl Iterator for Analysis<'_> {
    type Item = Tokens;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(x) => match x {
                Tokens::End => None,
                _ => Some(x),
            },
            Err(e) => {
                println!("{}", e);
                panic!("Lex error")
            }
        }
    }
}

impl<'a> Analysis<'a> {
    pub fn new(file: &'a str, source: &'a String) -> Self {
        let mut me = Analysis {
            file,
            iter_line: source.lines(),
            iter_c: source.chars(),
            line: "",
            line_offset: 1,
            c_offset: 0,
            buf: VecDeque::<Tokens>::with_capacity(512),
            peek: ' ',
        };
        me.line = match me.iter_line.next() {
            Some(x) => x,
            None => "",
        };
        me
    }

    pub fn new_with_capacity(file: &'a str, source: &'a String, capacity: usize) -> Self {
        let default = Analysis::new(file, source);
        Self {
            buf: VecDeque::<Tokens>::with_capacity(capacity),
            ..default
        }
    }

    fn readch(&mut self) {
        self.peek = match self.iter_c.next() {
            Some(c) => {
                if c == '\n' {
                    self.line = match self.iter_line.next() {
                        Some(x) => x,
                        None => EOF_STR,
                    };
                    self.line_offset += 1;
                    self.c_offset = 0;
                }
                self.c_offset += 1;
                c
            }
            None => EOF,
        }
    }

    fn expect(&mut self, expected: char) -> bool {
        self.readch();
        if self.peek != expected {
            return false;
        }
        self.peek = ' ';
        true
    }

    /// 从列表中匹配一个
    fn expect_or(&mut self, expecteds: Vec<char>) -> char {
        self.readch();
        for c in expecteds {
            if self.peek == c {
                self.peek = ' ';
                return c;
            }
        }
        self.peek
    }

    pub fn next_token(&mut self) -> Result<Tokens, Mis> {
        loop {
            if self.peek == ' ' || self.peek == '\n' {
                self.readch();
            } else if self.peek == EOF {
                return Ok(Tokens::End);
            } else {
                break;
            }
        }
        match self.peek {
            '\\' => {
                let mut e = Mis::new(
                    Froms::Lex,
                    Types::Error,
                    "转义符号外部使用",
                    self.file,
                    None,
                );
                e.add_line(
                    self.line_offset,
                    "转义符号只允许在字符串中使用",
                    self.line,
                    Some(LineType::Happen),
                    Some((self.c_offset, 1)),
                );
                return Err(e);
            }
            '"' => {
                let mut c = String::new();
                // 初始化，在发生字符串未闭合错误时保留信息
                let mut e = Mis::new(Froms::Lex, Types::Error, "字符串未闭合", self.file, None);
                e.add_line(
                    self.line_offset,
                    "字符串需在 \" \" 格式中",
                    self.line,
                    Some(LineType::Relate),
                    Some((self.c_offset, 1)),
                );
                loop {
                    match self.expect_or(vec!['\\', '"']) {
                        '\\' => match self.expect_or(vec!['t', 'n', '"', '\\']) {
                            't' => {
                                c.push('\t');
                            }
                            'n' => {
                                c.push('\n');
                            }
                            '"' => {
                                c.push('"');
                            }
                            '\\' => {
                                c.push('\\');
                            }
                            _ => {
                                let mut e = Mis::new(
                                    Froms::Lex,
                                    Types::Error,
                                    "错误转移符号使用",
                                    self.file,
                                    None,
                                );
                                e.add_line(
                                    self.line_offset,
                                    "转义符号使用不正当，未正确闭合",
                                    self.line,
                                    Some(LineType::Happen),
                                    Some((self.c_offset - 1, 2)),
                                );
                                return Err(e);
                            }
                        },
                        '"' => {
                            return Ok(Tokens::Str(c));
                        }
                        EOF => {
                            e.add_line(
                                self.line_offset,
                                "检测到字符串未闭合",
                                self.line,
                                Some(LineType::Happen),
                                Some((self.c_offset, 1)),
                            );
                            return Err(e);
                        }
                        other => {
                            c.push(other);
                        }
                    }
                }
            }
            '/' => {
                match self.expect_or(vec!['*', '/', '=']) {
                    '*' => {
                        let mut c = String::new();
                        // 先初始化，到发生未闭合错误时保留原本的信息
                        let mut e = Mis::new(
                            Froms::Preprocessor,
                            Types::Error,
                            "块注释未闭合",
                            self.file,
                            None,
                        );
                        e.add_line(
                            self.line_offset,
                            "块注释格式应为 /* ... */",
                            self.line,
                            Some(LineType::Relate),
                            Some((self.c_offset - 1, 2)),
                        );
                        loop {
                            match self.expect_or(vec!['*']) {
                                '*' => match self.expect('/') {
                                    true => {
                                        return Ok(Tokens::CommentBlock(c));
                                    }
                                    false => {
                                        c.push('*');
                                        c.push(self.peek);
                                    }
                                },
                                EOF => {
                                    e.add_line(
                                        self.line_offset,
                                        "检测到块注释未闭合",
                                        self.line,
                                        Some(LineType::Happen),
                                        Some((self.c_offset, 1)),
                                    );
                                    return Err(e);
                                }
                                other => {
                                    c.push(other);
                                }
                            }
                        }
                    }
                    '/' => {
                        if self.expect('!') {
                            let mut c = String::new();
                            loop {
                                match self.expect('\n') {
                                    true => {
                                        return Ok(Tokens::CommentModule(c));
                                    }
                                    false => {
                                        c.push(self.peek);
                                    }
                                }
                            }
                        } else {
                            let mut e = Mis::new(
                                Froms::Preprocessor,
                                Types::Error,
                                "无该符号: //",
                                self.file,
                                None,
                            );
                            e.add_line(
                                self.line_offset,
                                "词法分析检测到不应该出现的行注释，检查预处理器是否正常",
                                self.line,
                                Some(LineType::Happen),
                                Some((self.c_offset - 1, 2)),
                            );
                            return Err(e);
                        }
                    }
                    '=' => {
                        return Ok(Tokens::DivIs);
                    }
                    _ => {
                        return Ok(Tokens::Div);
                    }
                }
            }
            '+' => {
                if self.expect('=') {
                    return Ok(Tokens::PlusIs);
                } else {
                    return Ok(Tokens::Plus);
                }
            }
            '-' => match self.expect_or(vec!['=', '>']) {
                '=' => {
                    return Ok(Tokens::MinusIs);
                }
                '>' => {
                    return Ok(Tokens::ShouldReturn);
                }
                _ => {
                    return Ok(Tokens::Minus);
                }
            },
            '*' => {
                if self.expect('=') {
                    return Ok(Tokens::MulIs);
                } else {
                    return Ok(Tokens::Mul);
                }
            }
            '%' => {
                if self.expect('=') {
                    return Ok(Tokens::ModIs);
                } else {
                    return Ok(Tokens::Mod);
                }
            }
            '!' => {
                if self.expect('=') {
                    return Ok(Tokens::Ne);
                } else {
                    return Ok(Tokens::Negate);
                }
            }
            '>' => {
                if self.expect('=') {
                    return Ok(Tokens::Ge);
                } else {
                    return Ok(Tokens::Gt);
                }
            }
            '<' => {
                if self.expect('=') {
                    return Ok(Tokens::Le);
                } else {
                    return Ok(Tokens::Lt);
                }
            }
            '=' => {
                if self.expect('=') {
                    return Ok(Tokens::Eq);
                } else {
                    return Ok(Tokens::Is);
                }
            }
            '&' => {
                if self.expect('&') {
                    return Ok(Tokens::AndS);
                } else {
                    return Ok(Tokens::And);
                }
            }
            '|' => {
                if self.expect('|') {
                    return Ok(Tokens::OrS);
                } else {
                    return Ok(Tokens::Or);
                }
            }
            ';' => {
                self.peek = ' ';
                return Ok(Tokens::EndExp);
            }
            ':' => {
                self.peek = ' ';
                return Ok(Tokens::Semicolon);
            }
            ',' => {
                self.peek = ' ';
                return Ok(Tokens::Comma);
            }
            '.' => {
                self.peek = ' ';
                return Ok(Tokens::Dot);
            }
            '(' => {
                self.peek = ' ';
                return Ok(Tokens::LeftC);
            }
            ')' => {
                self.peek = ' ';
                return Ok(Tokens::RightC);
            }
            '{' => {
                self.peek = ' ';
                return Ok(Tokens::LeftBC);
            }
            '}' => {
                self.peek = ' ';
                return Ok(Tokens::RightBC);
            }
            '[' => {
                self.peek = ' ';
                return Ok(Tokens::LeftMB);
            }
            ']' => {
                self.peek = ' ';
                return Ok(Tokens::RightMB);
            }
            _ => (),
        }
        if self.peek.is_ascii_digit() {
            let mut c = String::new();
            let mut is_float = false;
            c.push(self.peek);
            loop {
                self.readch();
                if self.peek.is_ascii_digit() {
                    c.push(self.peek);
                } else if self.peek == '.' {
                    if is_float {
                        let mut e =
                            Mis::new(Froms::Lex, Types::Error, "浮点数格式错误", self.file, None);
                        e.add_line(
                            self.line_offset,
                            "浮点数中不应该出现多个 . 分割号",
                            self.line,
                            Some(LineType::Happen),
                            Some((self.c_offset - c.len(), c.len() + 1)),
                        );
                        return Err(e);
                    } else {
                        c.push(self.peek);
                        is_float = true;
                    }
                } else if self.peek == 'f' {
                    self.peek = ' ';
                    is_float = true;
                    break;
                } else if self.peek == 'u' {
                    self.peek = ' ';
                    is_float = false;
                    break;
                } else if !(self.peek.is_ascii_alphabetic() || self.peek == '_') {
                    break;
                } else {
                    let mut e = Mis::new(
                        Froms::Lex,
                        Types::Error,
                        "标识符禁止使用数字开头",
                        self.file,
                        None,
                    );
                    e.add_line(
                        self.line_offset,
                        "标识符禁止数字开头，常数应只在最后指定数据格式，且只能为限定的格式",
                        self.line,
                        Some(LineType::Happen),
                        Some((self.c_offset - c.len(), c.len() + 1)),
                    );
                    return Err(e);
                }
            }
            match is_float {
                true => match c.parse::<f32>() {
                    Ok(x) => {
                        return Ok(Tokens::Decimal(x));
                    }
                    Err(_) => {
                        let mut e =
                            Mis::new(Froms::Lex, Types::Error, "浮点数解析错误", self.file, None);
                        e.add_line(
                            self.line_offset,
                            "浮点数出错，检查格式以及强制类型指定是否正常",
                            self.line,
                            Some(LineType::Happen),
                            Some((self.c_offset - c.len() + 1, c.len())),
                        );
                        return Err(e);
                    }
                },
                false => match c.parse::<i32>() {
                    Ok(x) => {
                        return Ok(Tokens::Int(x));
                    }
                    Err(_) => {
                        let mut e =
                            Mis::new(Froms::Lex, Types::Error, "整常数解析错误", self.file, None);
                        e.add_line(
                            self.line_offset,
                            "整常数出错，检查格式以及强制类型指定是否正常",
                            self.line,
                            Some(LineType::Happen),
                            Some((self.c_offset - c.len() + 1, c.len())),
                        );
                        return Err(e);
                    }
                },
            }
        }
        if self.peek.is_ascii_alphabetic() || self.peek == '_' {
            let mut c = String::new();
            c.push(self.peek);
            loop {
                self.readch();
                if self.peek.is_ascii_alphanumeric() || self.peek == '_' {
                    c.push(self.peek);
                } else {
                    match c.as_ref() {
                        "true" => return Ok(Tokens::Bool(true)),
                        "false" => return Ok(Tokens::Bool(false)),
                        "if" => return Ok(Tokens::If),
                        "else" => return Ok(Tokens::Else),
                        "while" => return Ok(Tokens::While),
                        "loop" => return Ok(Tokens::Loop),
                        "break" => return Ok(Tokens::Break),
                        "match" => return Ok(Tokens::Match),
                        "fn" => return Ok(Tokens::Fn),
                        "struct" => return Ok(Tokens::Struct),
                        "return" => return Ok(Tokens::Return),
                        "let" => return Ok(Tokens::Let),
                        "mut" => return Ok(Tokens::Mut),
                        other => return Ok(Tokens::Identity(other.to_string())),
                    }
                }
            }
        }
        let mut e = Mis::new(Froms::Lex, Types::Error, "未知符号", self.file, None);
        e.add_line(
            self.line_offset,
            "出现未知符号，词法分析无法识别",
            self.line,
            Some(LineType::Happen),
            Some((self.c_offset, 1)),
        );
        Err(e)
    }
}

#[cfg(test)]
mod analysis_tests {
    use std::{fs::File, io::Write};

    use crate::lex::preprocessor::preprocessor;

    use super::*;

    macro_rules! check_tokens {
        ($analysis:expr, [$($tokens:expr),*]) => {
            $(
                assert_eq!($analysis, $tokens);
            )*
        };
    }

    #[test]
    fn test1() {
        let mut path = std::env::current_dir().unwrap();
        path.push("examples/sources/s1.ms");
        let file = File::open(path).unwrap();
        let s = preprocessor(&file);
        let mut analysis = Analysis::new_with_capacity("s1.ms", &s, s.len());
        check_tokens!(
            analysis.next_token().unwrap(),
            [
                Tokens::CommentModule(" This module is used to be a example code ".to_string()),
                Tokens::CommentModule(" * Note that it is ... ".to_string()),
                Tokens::CommentBlock(" \nBlock Comment \n* No use \n".to_string()),
                Tokens::Struct,
                Tokens::Identity("a".to_string()),
                Tokens::LeftBC,
                Tokens::Identity("b".to_string()),
                Tokens::Semicolon,
                Tokens::Identity("Type3".to_string()),
                Tokens::Comma,
                Tokens::Identity("c".to_string()),
                Tokens::Semicolon,
                Tokens::Identity("Type4".to_string()),
                Tokens::RightBC,
                Tokens::Fn,
                Tokens::Identity("main".to_string()),
                Tokens::LeftC,
                Tokens::Identity("param1".to_string()),
                Tokens::Semicolon,
                Tokens::Identity("Type1".to_string()),
                Tokens::Comma,
                Tokens::Identity("param2".to_string()),
                Tokens::Semicolon,
                Tokens::Identity("Type2".to_string()),
                Tokens::RightC,
                Tokens::ShouldReturn,
                Tokens::Identity("Identity_Type".to_string()),
                Tokens::LeftBC,
                Tokens::Let,
                Tokens::Mut,
                Tokens::Identity("_a".to_string()),
                Tokens::Semicolon,
                Tokens::Identity("Type3".to_string()),
                Tokens::Is,
                Tokens::Int(3),
                Tokens::EndExp,
                Tokens::Let,
                Tokens::Mut,
                Tokens::Identity("b3".to_string()),
                Tokens::Is,
                Tokens::Decimal(4.3),
                Tokens::EndExp,
                Tokens::Let,
                Tokens::Identity("cons".to_string()),
                Tokens::Is,
                Tokens::Bool(true),
                Tokens::EndExp,
                Tokens::Let,
                Tokens::Identity("s".to_string()),
                Tokens::Is,
                Tokens::Str("Nihao".to_string()),
                Tokens::EndExp,
                Tokens::If,
                Tokens::Identity("_a".to_string()),
                Tokens::Gt,
                Tokens::Identity("b3".to_string()),
                Tokens::LeftBC,
                Tokens::Identity("_a".to_string()),
                Tokens::Is,
                Tokens::Int(4),
                Tokens::EndExp,
                Tokens::Identity("print".to_string()),
                Tokens::LeftC,
                Tokens::Str("".to_string()),
                Tokens::RightC,
                Tokens::EndExp,
                Tokens::RightBC,
                Tokens::Else,
                Tokens::If,
                Tokens::LeftC,
                Tokens::Identity("_a".to_string()),
                Tokens::Lt,
                Tokens::Identity("b3".to_string()),
                Tokens::RightC,
                Tokens::LeftBC,
                Tokens::Identity("print".to_string()),
                Tokens::LeftC,
                Tokens::Str("Hello".to_string()),
                Tokens::RightC,
                Tokens::EndExp,
                Tokens::RightBC,
                Tokens::Else,
                Tokens::If,
                Tokens::LeftC,
                Tokens::Identity("_a".to_string()),
                Tokens::Eq,
                Tokens::Identity("b3".to_string()),
                Tokens::RightC,
                Tokens::LeftBC,
                Tokens::Identity("print".to_string()),
                Tokens::LeftC,
                Tokens::Str("cd".to_string()),
                Tokens::RightC,
                Tokens::EndExp,
                Tokens::RightBC,
                Tokens::While,
                Tokens::Identity("_a".to_string()),
                Tokens::Gt,
                Tokens::Int(3),
                Tokens::OrS,
                Tokens::Identity("cons".to_string()),
                Tokens::Eq,
                Tokens::Bool(false),
                Tokens::LeftBC,
                Tokens::Identity("a".to_string()),
                Tokens::Is,
                Tokens::Identity("a".to_string()),
                Tokens::Minus,
                Tokens::Int(1),
                Tokens::EndExp,
                Tokens::RightBC,
                Tokens::LeftBC,
                Tokens::Identity("b3".to_string()),
                Tokens::Is,
                Tokens::Identity("_a".to_string()),
                Tokens::Minus,
                Tokens::Int(1),
                Tokens::EndExp,
                Tokens::If,
                Tokens::Identity("_a".to_string()),
                Tokens::Gt,
                Tokens::Int(3),
                Tokens::LeftBC,
                Tokens::Break,
                Tokens::EndExp,
                Tokens::RightBC,
                Tokens::RightBC,
                Tokens::Loop,
                Tokens::Identity("b3".to_string()),
                Tokens::Le,
                Tokens::Int(3),
                Tokens::Match,
                Tokens::Identity("cons".to_string()),
                Tokens::LeftBC,
                Tokens::Bool(true),
                Tokens::Semicolon,
                Tokens::LeftBC,
                Tokens::Return,
                Tokens::EndExp,
                Tokens::RightBC,
                Tokens::Comma,
                Tokens::Bool(false),
                Tokens::Semicolon,
                Tokens::LeftBC,
                Tokens::Return,
                Tokens::EndExp,
                Tokens::RightBC,
                Tokens::RightBC,
                Tokens::Return,
                Tokens::Int(3),
                Tokens::EndExp,
                Tokens::RightBC,
                Tokens::End
            ]
        );
    }

    #[test]
    fn test2() {
        let mut path = std::env::current_dir().unwrap();
        path.push("examples/sources/s2.ms");
        let file = File::open(path).unwrap();
        let s = preprocessor(&file);
        let mut analysis = Analysis::new_with_capacity("s1.ms", &s, s.len());
        check_tokens!(
            analysis.next_token().unwrap(),
            [
                Tokens::Fn,
                Tokens::Identity("main".to_string()),
                Tokens::LeftC,
                Tokens::Identity("c".to_string()),
                Tokens::Semicolon,
                Tokens::LeftMB,
                Tokens::Identity("int".to_string()),
                Tokens::EndExp,
                Tokens::Int(8),
                Tokens::RightMB,
                Tokens::Comma,
                Tokens::Identity("t".to_string()),
                Tokens::Semicolon,
                Tokens::Identity("float".to_string()),
                Tokens::RightC,
                Tokens::ShouldReturn,
                Tokens::LeftMB,
                Tokens::Identity("float".to_string()),
                Tokens::EndExp,
                Tokens::Int(10),
                Tokens::RightMB,
                Tokens::LeftBC,
                Tokens::Let,
                Tokens::Mut,
                Tokens::Identity("a".to_string()),
                Tokens::Semicolon,
                Tokens::Identity("int".to_string()),
                Tokens::Is,
                Tokens::Minus,
                Tokens::Int(4),
                Tokens::EndExp,
                Tokens::Let,
                Tokens::Identity("_b33_a".to_string()),
                Tokens::Is,
                Tokens::Decimal(4f32),
                Tokens::EndExp,
                Tokens::Identity("a".to_string()),
                Tokens::PlusIs,
                Tokens::Identity("_b33_a".to_string()),
                Tokens::EndExp,
                Tokens::Identity("a".to_string()),
                Tokens::DivIs,
                Tokens::Identity("_b33_a".to_string()),
                Tokens::EndExp,
                Tokens::Identity("a".to_string()),
                Tokens::MinusIs,
                Tokens::Identity("_b33_a".to_string()),
                Tokens::EndExp,
                Tokens::Identity("a".to_string()),
                Tokens::MulIs,
                Tokens::Identity("_b33_a".to_string()),
                Tokens::EndExp,
                Tokens::Identity("a".to_string()),
                Tokens::ModIs,
                Tokens::Identity("_b33_a".to_string()),
                Tokens::EndExp,
                Tokens::If,
                Tokens::LeftC,
                Tokens::LeftC,
                Tokens::Identity("a".to_string()),
                Tokens::Lt,
                Tokens::Identity("_b33_a".to_string()),
                Tokens::RightC,
                Tokens::Or,
                Tokens::Identity("a".to_string()),
                Tokens::Ge,
                Tokens::Identity("_b33_a".to_string()),
                Tokens::RightC,
                Tokens::OrS,
                Tokens::LeftC,
                Tokens::LeftC,
                Tokens::Identity("a".to_string()),
                Tokens::Eq,
                Tokens::Identity("_b33_a".to_string()),
                Tokens::RightC,
                Tokens::And,
                Tokens::Identity("a".to_string()),
                Tokens::Ne,
                Tokens::Identity("_b33_a".to_string()),
                Tokens::RightC,
                Tokens::AndS,
                Tokens::Identity("a".to_string()),
                Tokens::LeftBC,
                Tokens::Return,
                Tokens::Bool(true),
                Tokens::EndExp,
                Tokens::RightBC,
                Tokens::Return,
                Tokens::Bool(false),
                Tokens::EndExp,
                Tokens::RightBC,
                Tokens::End
            ]
        );
    }

    #[test]
    fn test3() {
        let mut path = std::env::current_dir().unwrap();
        path.push("examples/sources/s3.ms");
        let file = File::open(path).unwrap();
        let s = preprocessor(&file);
        let mut analysis = Analysis::new_with_capacity("s1.ms", &s, s.len());
        check_tokens!(
            analysis.next_token().unwrap(),
            [
                Tokens::CommentBlock("*It is a wrong program ".to_string()),
                Tokens::Fn,
                Tokens::Identity("main".to_string()),
                Tokens::LeftC,
                Tokens::RightC,
                Tokens::LeftBC,
                Tokens::Let
            ]
        );
        let x = analysis.next_token();
        assert!(x.is_err());
        if let Some(e) = x.err() {
            println!("{}", e);
        }
    }

    #[test]
    fn test4() {
        let mut path = std::env::current_dir().unwrap();
        path.push("examples/sources/s4.ms");
        let file = File::open(path).unwrap();
        let s = preprocessor(&file);
        let mut analysis = Analysis::new_with_capacity("s1.ms", &s, s.len());
        check_tokens!(
            analysis.next_token().unwrap(),
            [
                Tokens::CommentBlock(" \nThis is another wrong program ".to_string()),
                Tokens::Struct,
                Tokens::Identity("b".to_string()),
                Tokens::LeftBC,
                Tokens::Identity("d".to_string()),
                Tokens::Semicolon,
                Tokens::LeftMB,
                Tokens::Identity("int".to_string()),
                Tokens::EndExp,
                Tokens::Int(10),
                Tokens::RightMB,
                Tokens::RightBC,
                Tokens::Fn,
                Tokens::Identity("main".to_string()),
                Tokens::LeftC,
                Tokens::Identity("a".to_string()),
                Tokens::Semicolon,
                Tokens::LeftC,
                Tokens::Identity("b".to_string()),
                Tokens::Comma,
                Tokens::Identity("int".to_string()),
                Tokens::RightC,
                Tokens::RightC,
                Tokens::ShouldReturn,
                Tokens::Identity("int".to_string()),
                Tokens::LeftBC,
                Tokens::Identity("print".to_string()),
                Tokens::LeftC,
                Tokens::Identity("b".to_string()),
                Tokens::Dot,
                Tokens::Identity("d".to_string()),
                Tokens::RightC,
                Tokens::EndExp,
                Tokens::Let,
                Tokens::Identity("a".to_string()),
                Tokens::Is
            ]
        );
        let x = analysis.next_token();
        assert!(x.is_err());
        if let Some(e) = x.err() {
            println!("{}", e);
        }
    }

    #[test]
    fn test5() {
        let mut path = std::env::current_dir().unwrap();
        path.push("examples/sources/s5.ms");
        let file = File::open(path).unwrap();
        let s = preprocessor(&file);
        let mut analysis = Analysis::new_with_capacity("s1.ms", &s, s.len());
        check_tokens!(
            analysis.next_token().unwrap(),
            [
                Tokens::Fn,
                Tokens::Identity("main".to_string()),
                Tokens::LeftC,
                Tokens::RightC,
                Tokens::LeftBC
            ]
        );
        let x = analysis.next_token();
        assert!(x.is_err());
        if let Some(e) = x.err() {
            println!("{}", e);
        }
    }

    #[test]
    fn test6() {
        let mut path = std::env::current_dir().unwrap();
        path.push("examples/sources/s6.ms");
        let file = File::open(path).unwrap();
        let s = preprocessor(&file);
        let mut analysis = Analysis::new_with_capacity("s1.ms", &s, s.len());
        check_tokens!(
            analysis.next_token().unwrap(),
            [
                Tokens::Fn,
                Tokens::Identity("main".to_string()),
                Tokens::LeftC,
                Tokens::RightC,
                Tokens::LeftBC,
                Tokens::Let,
                Tokens::Mut,
                Tokens::Identity("s".to_string()),
                Tokens::Is,
                Tokens::Str("\n\t\" Hello World.!= \" \\ ".to_string()),
                Tokens::EndExp,
                Tokens::Let,
                Tokens::Mut,
                Tokens::Identity("v".to_string()),
                Tokens::Is
            ]
        );
        let x = analysis.next_token();
        assert!(x.is_err());
        if let Some(e) = x.err() {
            println!("{}", e);
        }
    }

    #[test]
    fn test7() {
        let mut path = std::env::current_dir().unwrap();
        path.push("examples/sources/s7.ms");
        let file = File::open(path).unwrap();
        let s = preprocessor(&file);
        let mut analysis = Analysis::new_with_capacity("s1.ms", &s, s.len());
        check_tokens!(
            analysis.next_token().unwrap(),
            [
                Tokens::Fn,
                Tokens::Identity("main".to_string()),
                Tokens::LeftC,
                Tokens::RightC,
                Tokens::LeftBC
            ]
        );
        let x = analysis.next_token();
        assert!(x.is_err());
        if let Some(e) = x.err() {
            println!("{}", e);
        }
    }

    #[test]
    fn test8() {
        let mut source = std::env::current_dir().unwrap();
        source.push("examples/sources/s1.ms");
        let mut test = std::env::current_dir().unwrap();
        test.push("examples/tests/");
        // 创建目录
        if !&test.exists() {
            std::fs::create_dir(&test).unwrap();
        }
        test.push("test_serize.txt");
        let fs = File::open(source).unwrap();
        let s = preprocessor(&fs);
        let mut analysis = Analysis::new_with_capacity("s1.ms", &s, s.len());
        // 以等长的二元组形式输出至文件
        let mut ft = File::create(test).unwrap();
        loop {
            let x = analysis.next_token().unwrap();
            if x != Tokens::End {
                ft.write_all(format!("{}\n", x.dump()).as_bytes()).unwrap();
            } else {
                break;
            }
        }
    }
}
