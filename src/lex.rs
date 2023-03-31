//! 词法分析器

pub mod analysis;
pub mod preprocessor;

/// 词法单元，二元组（词法类型 + 词法值）
/// 区分大小写
/// 注释：
/// 1. // 行注释（一行），值要带上注释（在预处理中去掉，不作为正式语法单元）
/// 2. /* */ 块注释（跨行），值要带上注释
/// 3. //! 模块注释（文件，每次一行），值要带上注释
/// 运算符号：
/// 1. + - * / % ! > < = += -= *= /= %= >= <= == != & && | || 
/// 分隔符：
/// 1. ; , : . () {} []
/// 字符串常量：
/// 1. 形式1："xxx" ，值要带上里面的字符串，\ 不作为单独的词法符号处理，针对识别字符串常量时的特殊符号
/// 整常数
/// 1. 形式为 4 / 4u
/// 小数
/// 1. 形式为 4.3 / 4f / 4.3f
/// 布尔常数
/// 1. true false
/// 标识符
/// 1. 变量名、函数、过程名、类型名（非数字开头以字母或者_开头，非开头中可由字母、_符号、数字组成）
/// 关键字
/// 1. if else while loop break match fn struct
/// 2. ->（函数返回值，用于定义中） return（函数体内返回） let mut
#[derive(PartialEq, Debug, Clone)]
pub enum Tokens {
    /// 块注释
    CommentBlock(String),
    /// 模块注释
    CommentModule(String),
    /// 字符串常量
    Str(String),
    /// 加
    Plus,
    /// 减
    Minus,
    /// 乘
    Mul,
    /// 除
    Div,
    /// 取余
    Mod,
    /// 取反 !
    Negate,
    /// \>
    Gt,
    /// <
    Lt,
    /// \>=
    Ge,
    /// <=
    Le,
    /// ==
    Eq,
    /// !=
    Ne,
    /// +=
    PlusIs,
    /// -=
    MinusIs,
    /// /=
    DivIs,
    /// *=
    MulIs,
    /// %=
    ModIs,
    /// =
    Is,
    /// &
    And,
    /// &&
    AndS,
    /// |
    Or,
    /// ||
    OrS,
    /// ;
    EndExp,
    /// ,
    Comma,
    /// :
    Semicolon,
    /// .
    Dot,
    /// (
    LeftC,
    /// )
    RightC,
    /// {
    LeftBC,
    /// }
    RightBC,
    /// [
    LeftMB,
    /// ]
    RightMB,
    /// 整常数
    Int(i32),
    /// 小数
    Decimal(f32),
    /// 布尔常数
    Bool(bool),
    /// 标识符
    Identity(String),
    If,
    Else,
    While,
    Loop,
    Break,
    Match,
    /// 函数关键字
    Fn,
    /// 结构体关键字
    Struct,
    Let,
    Mut,
    /// 函数返回值，定义中
    ShouldReturn,
    /// 函数返回值，函数体中
    Return,
    /// 源程序读入结束标志（也可以作为语法的开始标志）
    End,
    /// 空
    Null
}

impl Tokens {
    pub fn dump(&self) -> String {
        match self {
            Tokens::CommentBlock(x) => format!("(Comment Block, \"{}\")", x),
            Tokens::CommentModule(x) => format!("(Comment Module, \"{}\")", x),
            Tokens::Str(x) => format!("(String, \"{}\")", x),
            Tokens::Plus => "(Plus, )".to_string(),
            Tokens::Minus => "(Minus, )".to_string(),
            Tokens::Mul => "(Mul, )".to_string(),
            Tokens::Div => "(Div, )".to_string(),
            Tokens::Mod => "(Mod, )".to_string(),
            Tokens::Negate => "(Negate, )".to_string(),
            Tokens::Gt => "(Gt, )".to_string(),
            Tokens::Lt => "(Lt, )".to_string(),
            Tokens::Ge => "(Ge, )".to_string(),
            Tokens::Le => "(Le, )".to_string(),
            Tokens::Eq => "(Eq, )".to_string(),
            Tokens::Ne => "(Ne, )".to_string(),
            Tokens::PlusIs => "(PlusIs, )".to_string(),
            Tokens::MinusIs => "(MinusIs, )".to_string(),
            Tokens::DivIs => "(DivIs, )".to_string(),
            Tokens::MulIs => "(MulIs, )".to_string(),
            Tokens::ModIs => "(ModIs, )".to_string(),
            Tokens::Is => "(Is, )".to_string(),
            Tokens::And => "(And, )".to_string(),
            Tokens::AndS => "(AndS, )".to_string(),
            Tokens::Or => "(Or, )".to_string(),
            Tokens::OrS => "(OrS, )".to_string(),
            Tokens::EndExp => "(EndExp, )".to_string(),
            Tokens::Comma => "(Comma, )".to_string(),
            Tokens::Semicolon => "(Semicolon, )".to_string(),
            Tokens::Dot => "(Dot, )".to_string(),
            Tokens::LeftC => "(LeftC, )".to_string(),
            Tokens::RightC => "(RightC, )".to_string(),
            Tokens::LeftBC => "(LeftBC, )".to_string(),
            Tokens::RightBC => "(RightBC, )".to_string(),
            Tokens::LeftMB => "(LeftMB, )".to_string(),
            Tokens::RightMB => "(RightMB, )".to_string(),
            Tokens::Int(x) => format!("(Int, \"{}\")", x),
            Tokens::Decimal(x) => format!("(Decimal, \"{}\")", x),
            Tokens::Bool(x) => format!("(Bool, \"{}\")", x),
            Tokens::Identity(x) => format!("(Identity, \"{}\")", x),
            Tokens::If => "(If, )".to_string(),
            Tokens::Else => "(Else, )".to_string(),
            Tokens::While => "(While, )".to_string(),
            Tokens::Loop => "(Loop, )".to_string(),
            Tokens::Break => "(Break, )".to_string(),
            Tokens::Match => "(Match, )".to_string(),
            Tokens::Fn => "(Fn, )".to_string(),
            Tokens::Struct => "(Struct, )".to_string(),
            Tokens::Let => "(Let, )".to_string(),
            Tokens::Mut => "(Mut, )".to_string(),
            Tokens::ShouldReturn => "(ShouldReturn, )".to_string(),
            Tokens::Return => "(Return, )".to_string(),
            Tokens::End => "(End, )".to_string(),
            Tokens::Null => "(ε, )".to_string()
        }
    }
}