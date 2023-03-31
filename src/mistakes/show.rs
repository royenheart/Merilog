use std::fmt::{Display, Formatter};

/// 来源
#[derive(PartialEq, Debug)]
pub enum Froms {
    Preprocessor,
    Lex,
    Syntax,
    Semantic
}

impl Display for Froms {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Froms::Preprocessor => write!(f, "预处理阶段"),
            Froms::Lex => write!(f, "词法分析阶段"),
            Froms::Syntax => write!(f, "语法分析阶段"),
            Froms::Semantic => write!(f, "语义分析阶段"),
        }
    }
}

/// 类型
#[derive(PartialEq, Debug)]
pub enum Types {
    Info,
    Warning,
    Error
}

impl Display for Types {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Types::Info => write!(f, "信息"),
            Types::Warning => write!(f, "警告"),
            Types::Error => write!(f, "错误"),
        }
    }
}

/// 各行的类型
#[derive(PartialEq, Debug)]
pub enum LineType {
    Note,
    Happen,
    Relate
}

/// 错误类型
/// 1. 分析出错行数和偏移量
/// 2. 发生错误的位置和报错
/// 3. 和错误有关的位置的提示（往往是引起错误的，或者是一些帮助修改的提示性信息）
#[derive(PartialEq, Debug)]
pub struct Mis<'a> {
    who: Froms, 
    /// 类型
    wtype: Types,
    /// 介绍
    intro: &'a str,
    file: &'a str,
    /// 行号，各行类型，起始位置，偏移量，提示信息，具体代码
    lines: Option<Vec<(usize, Option<LineType>, Option<(usize, usize)>, &'a str, &'a str)>>,
}

impl<'a> Mis<'a> {
    pub fn new (who: Froms, wtype: Types, intro: &'a str, file: &'a str, lines: Option<Vec<(usize, Option<LineType>, Option<(usize, usize)>, &'a str, &'a str)>>) -> Self {
        Mis { who, wtype, intro, file, lines }
    }

    pub fn add_line(&mut self, line: usize, info: &'a str, code: &'a str, line_type: Option<LineType>, pos: Option<(usize, usize)>) {
        let g = (line, line_type, pos, info, code);
        match &mut self.lines {
            Some(x) => x.push(g),
            None => {
                self.lines = Some(vec![g])
            }
        }
    }
}

impl<'a> Display for Mis<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // 出错行号
        let mut l = String::new();
        // 提示信息
        let mut s = String::new();
        let width: usize = 6;
        if let Some(x) = &self.lines {
            s.push_str(format!("{:width$}|\n", ' ', width = width).as_str());
            for (line, line_type, pos, info, code) in x {
                let start_from: usize;
                let how_many: usize;
                let stand;
                l.push_str(format!("{}", line).as_str());
                match *pos {
                    Some(x) => {
                        let (start, offset) = x;
                        l.push_str(format!(":{}", start).as_str());
                        start_from = start - 1;
                        how_many = offset;
                    },
                    None => {
                        start_from = 0;
                        how_many = 1;
                    }
                }
                l.push(';');
                match line_type {
                    Some(x) => {
                        match x {
                            LineType::Note => stand = '*',
                            LineType::Happen => stand = '^',
                            LineType::Relate => stand = '&'
                        }
                    },
                    None => {
                        stand = '-'
                    }
                }
                s.push_str(format!("{:width$}|\t{}\n", line, code, width=6).as_str());
                s.push_str(format!("{:width$}|\t{:from$}", "", "", width = width, from = start_from).as_str());
                for _ in 1..=how_many {
                    s.push(stand);
                }
                s.push_str(format!("-->{}\n", info).as_str());
            }
        }
        write!(f, "{} from {}: {}\n In {}:{}\n{}", self.wtype, self.who, self.intro, self.file, l, s)
    }
}