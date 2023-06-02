//! 语法分析

// 2023.6.6，目前文法还不能识别 !!true 这种多重单目运算。。。
// ExecExpSigOp -> SigOps ... 这块需要多重定义一下。。。

pub mod ll_parser;

use std::{hash::Hash, fmt::{Debug, Display}, borrow::BorrowMut};

use id_tree::Tree;
use id_tree_layout::Visualize;

use crate::lex::Tokens;

// 基于表达式的语言
// 基本运算处理（算数表达式、布尔表达式、字符串操作）、声明语句处理（类型、变量、数组、结构体）、结构体定义和访问、执行语句处理（赋值、选择 match、循环 while 和 loop 以及循环退出 break、语句块、函数调用）、函数定义处理（fn xxx(xxx) -> xxx {}）、作用域？（可能语义）、可变与非可变？（可能语义）、模块注释处理
// 错误处理以及警告（warning，不直接退出）
// 划分子任务
// 结构化程序设计（将不同的算法分模块）或其他可行设计思路
// 最终输出（最根本的目的）是 AST 抽象语法树

// 支持结构体，可组合成符合类型（数组、元组）

// Comment 注释语句可以不管（词法提取出来保存，后面带位置信息，可以跟语法树进行对比插入处理）
// 分成一个额外的模块即可
// 实现可变不可变借用
// 实现作用域
// 实现 match 语句
// 语言的特性是什么
// Tokens 枚举为终结符，其余均为非终结符
// 语句块（也相当于函数内语句块，最后一个可以为单纯的表达式，表示返回值）
// 强类型，表达式有类型，语句的类型是()，即空元组

/* 

开始符号为 Merilog
源文件以结构体声明、函数为基本单位
Merilog -> DefineStruct Merilog | DefineFn Merilog | Tokens::Null

First(Merilog) = {Tokens::Struct, Tokens::Fn, Tokens::Null}

Follow(Merilog) = {Tokens::End}

*/

/* 结构体声明

结构体内可声明成员和函数，之间必须都通过逗号进行分割
DefineStruct -> Tokens::Struct Tokens::Identity Tokens::LeftBC DefineStructBody Tokens::RightBC
DefineStructBody -> Tokens::Identity Tokens::Semicolon ExecType DefineStructBodyNext | DefineFn DefineStructBodyNext
DefineStructBodyNext -> Tokens::Comma DefineStructBody | Tokens::Null

First(DefineStruct) = {Tokens::Struct}
First(DefineStructBody) = {Tokens::Identity, Tokens::Fn}
First(DefineStructBodyNext) = {Tokens::Comma, Tokens::Null}

Follow(DefineStruct) = {Tokens::End}
Follow(DefineStructBody) = {Tokens::RightBC}
Follow(DefineStructBodyNext) = {Tokens::RightBC}

*/

/* 基本声明语句（类型、变量、数组、结构体、元组）

DefineVar -> Tokens::let DefineVarMutable
支持
DefineVarMutable -> Tokens::mut DefineVarS | DefineVarS
DefineVarS -> Tokens::Identity DefineVarType DefineVarValue DefineVarE
支持自动类型推导，并支持复合类型声明
DefineVarType -> Tokens::Null | Tokens::Semicolon ExecType
可不初始进行赋值
DefineVarValue -> Tokens::Null | Tokens::Is ExecExp
可声明多次
DefineVarE -> Tokens::Comma DefineVarS | Tokens::Null

First(DefineVar) = {Tokens::let}
First(DefineVarMutable) = {Tokens::mut, Tokens::Identity}
First(DefineVarS) = {Tokens::Identity} 
First(DefineVarType) = {Tokens::Semicolon， Tokens::Null}
First(DefineVarValue) = {Tokens::Is, Tokens::Null}
First(DefineVarE) = {Tokens::Comma, Tokens::Null}

Follow(DefineVar) = {Tokens::EndExp}
Follow(DefineVarMutable) = {Tokens::EndExp}
Follow(DefineVarS) = {Tokens::EndExp}
Follow(DefineVarType) = {Tokens::Is, Tokens::Comma, Tokens::EndExp}
Follow(DefineVarValue) = {Tokens::Comma, Tokens::EndExp}
Follow(DefineVarE) = {Tokens::EndExp}

*/

/* 执行语句（基于表达式，分语句和表达式，包含基本运算处理、基本声明语句）。有些结构作为语句（比如赋值），有些作为表达式。

可作为表达式的：

1. 基本运算语句
2. match 选择
3. 成员引用（复合类型引用和结构体引用）
4. 函数调用和结构体函数调用（本身算在成员引用中）

可作为语句的

1. If 条件判断语句
2. While / Loop 循环语句
3. 赋值语句、声明语句
4. 返回、break语句

语句（本身也可以算特殊表达式，返回 () 空元组）
ExecSentence -> ExecStmt Tokens::EndExp | ExecIs Tokens::EndExp | ExecIf | ExecWhile | ExecLoop Tokens::EndExp | ExecRet Tokens::EndExp | ExecBreak Tokens::EndExp

First(ExecSentence) = {Tokens::Let, Tokens::Identity, Tokens::If, Tokens::While, Tokens::LeftBC, Tokens::Return, Tokens::Break}

Follow(ExecSentence) = {Tokens::Let, Tokens::Identity, Tokens::If, Tokens::While, Tokens::LeftBC, Tokens::Return, Tokens::Break, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::RightBC}

表达式

    + - * / % ! > < >= <= == != & && | || ()
    优先级关系：
    1. ()
    2. ! + -
    3. * / %
    4. + -
    5. > >= < <=
    6. == !=
    7. &
    8. |
    10. &&
    11. ||

ExecExp -> ExecExpAndS R1
R1 -> Tokens::OrS ExecExp R1 | Tokens::Null
ExecExpAndS -> ExecExpOr R2
R2 -> Tokens::AndS ExecExpAndS R2 | Tokens::Null
ExecExpOr -> ExecExpAnd R3
R3 -> Tokens::Or ExpecExpOr R3 | Tokens::Null
ExecExpAnd -> ExecExpEq R4
R4 -> Tokens::And ExecExpAnd R4 | Tokens::Null
ExecExpEq -> ExecExpLGq R5
R5 -> Eqs ExecExpEq R5 | Tokens::Null
ExecExpLGq -> ExecExpAddOp R6
R6 -> LGqs ExecExpLGq R6 | Tokens::Null
ExecExpAddOp -> ExecExpMultiOp R7
R7 -> AddOps ExecExpAddOp R7 | Tokens::Null
ExecExpMultiOp -> ExecExpSigOp R8
R8 -> MultiOps ExecExpMultiOp R8 | Tokens::Null
ExecExpSigOp -> SigOps ExecExpN
ExecExpN -> Ops | Tokens::LeftC ExecExp Tokens::RightC
Eqs -> Tokens::Eq | Tokens::Ne
LGqs -> Tokens::Gt | Tokens::Lt | Tokens::Ge | Tokens::Le
AddOps -> Tokens::Plus | Tokens::Minus
MultiOps -> Tokens::Mul | Tokens::Div | Tokens::Mod
SigOps -> Tokens::Negate | Tokens::Plus | Tokens::Minus | Tokens::Null
Ops -> Tokens::Str | Tokens::Int | Tokens::Decimal | Tokens::Bool | ExecMatch | ExecVar

First(ExecExp) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
First(R1) = {Tokens::OrS, Tokens::Null}
First(ExecExpAndS) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
First(R2) = {Tokens::AndS, Tokens::Null}
First(ExecExpOr) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
First(R3) = {Tokesns::Or, Tokens::Null}
First(ExecExpAnd) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
First(R4) = {Tokens::And, Tokens::Null}
First(ExecExpEq) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
First(R5) = {Tokens::Eq, Tokens::Ne, Tokens::Null}
First(ExecExpLGq) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
First(R6) = {Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Null}
First(ExecExpAddOp) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
First(R7) = {Tokens::Plus, Tokens::Minus, Tokens::Null}
First(ExecExpMultiOp) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
First(R8) = {Tokens::Mul, Tokens::Div, Tokens::Mod, Tokens::Null}
First(ExecExpSigOp) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
First(ExecExpN) = {Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
First(Eqs) = {Tokens::Eq, Tokens::Ne}
First(LGqs) = {Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le}
First(AddOps) = {Tokens::Plus, Tokens::Minus}
First(MultiOps) = {Tokens::Mul, Tokens::Div, Tokens::Mod}
First(SigOps) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Null}
First(Ops) = {Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity}

Follow(ExecExp) = {Tokens::RightC, Tokens::OrS, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(R1) = {Tokens::RightC, Tokens::OrS, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecExpAndS) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(R2) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecExpOr) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(R3) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecExpAnd) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(R4) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecExpEq) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(R5) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecExpLGq) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(R6) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecExpAddOp) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(R7) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecExpMultiOp) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div, Tokens::Mod, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(R8) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div, Tokens::Mod, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecExpSigOp) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div, Tokens::Mod, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecExpN) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div, Tokens::Mod, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(Eqs) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
Follow(LGqs) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
Follow(AddOps) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
Follow(MultiOps) = {Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
Follow(SigOps) = {Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC}
Follow(Ops) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div, Tokens::Mod, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}

声明
ExecStmt -> DefineVar

First(ExecStmt) = {Tokens::Let}

Follow(ExecStmt) = {Tokens::EndExp}

返回
ExecRet -> Tokens::Return ExecExp

First(ExecRet) = {Tokens::Return}

Follow(ExecRet) = {Tokens::EndExp}

break
ExecBreak -> Tokens::Break

First(ExecBreak) = {Tokens::Break}

Follow(ExecBreak) = {Tokens::EndExp}

赋值语句（复合类型可被修改）
可选赋值符号
+= -= /= *= %= =
ExecIS -> ExecVar ExecIsW ExecExp
ExecIsW -> Tokens::PlusIs | Tokens::MinusIs | Tokens::DivIs | Tokens::MulIs | Tokens::ModIs | Tokens::Is

First(ExecIS) = {Tokens::Identity}
First(ExecIsW) = {Tokens::PlusIs, Tokens::MinusIs, Tokens::DivIS, Tokens::MulIs, Tokens::ModIs, Tokens::Is}

Follow(ExecIS) = {Tokens::EndExp}
Follow(ExecIsW) = {Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus}

Match 选择语句
ExecMatch -> Tokens::Match ExecExp Tokens::LeftBC ExecMatchS Tokens::RightBC
ExecMatchS -> ExecExp Tokens::Semicolon Tokens::LeftBC FnBody Tokens::RightBC ExecMatchE
ExecMatchE -> Tokens::Comma ExecMatchS | Tokens::Null

First(ExecMatch) = {Tokens::Match}
First(ExecMatchS) = {Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus}
First(ExecMatchE) = {Tokens::Comma, Tokens::Null}

Follow(ExecMatch) = {Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div, Tokens::Mod}
Follow(ExecMatchS) = {Tokens::RightBC}
Follow(ExecMatchE) = {Tokens::RightBC}

If 条件判断语句
在语义分析阶段进行 Else 位置的判断
ExecIf -> Tokens::If ExecExp Tokens::LeftBC FnBody Tokens::RightBC ExecIfE
ExecIfE -> Tokens::Else ExecIfEi Tokens::LeftBC FnBody Tokens::RightBC ExecIfE | Tokens::Null
ExecIfEi -> Tokens::If ExecExp | Tokens::Null

First(ExecIf) = {Tokens::If}
First(ExecIfEi) = {Tokens::Else, Tokens::Null}
First(ExecIfE) = {Tokens::If, Tokens::Null}

Follow(ExecIf) = {Tokens::Let, Tokens::Identity, Tokens::If, Tokens::While, Tokens::LeftBC, Tokens::Return, Tokens::Break, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::RightBC}
Follow(ExecIfE) = {Tokens::Let, Tokens::Identity, Tokens::If, Tokens::While, Tokens::LeftBC, Tokens::Return, Tokens::Break, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::RightBC}
Follow(ExecIfEi) = {Tokens::LeftBC}

While 循环
ExecWhile -> Tokens::While ExecExp Tokens::LeftBC FnBody Tokens::RightBC

First(ExecWhile) = {Tokens::While}

Follow(ExecWhile) = {Tokens::Let, Tokens::Identity, Tokens::If, Tokens::While, Tokens::LeftBC, Tokens::Return, Tokens::Break, Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::RightBC}

Loop 循环
ExecLoop -> Tokens::LeftBC FnBody Tokens::RightBC Tokens::Loop ExecExp

First(ExecLoop) = {Tokens::LeftBC}

Follow(ExecLoop) = {Tokens::EndExp}

成员引用（成员可能是单独的变量，也有可能是数组成员，也可能是元组，也可以是结构体，也可能是函数）
引用示例：
1. 变量： var
2. 数组成员：var[1]
3. 元组成员：var->0
4. 结构体：var->field1
5. 函数：var(...)
可以嵌套使用：
1. var[1]->0->field1
2. var(...)[1] - 后面语义分析的时候可能需要进行确认合法性
元组类型是 (Type1, Type2, ...)

ExecVar -> Tokens::Identity ExecVarT
ExecVarT -> Tokens::LeftMB Tokens::Int Tokens::RightMB ExecVarT | Tokens::ShouldReturn ExecVarSoE ExecVarT | Tokens::Null | Tokens::LeftC ExecFuncP Tokens::RightC ExecVarT
ExecVarSoE -> Tokens::Int | Tokens::Identity
ExecFuncP -> Tokens::Null | ExecFuncParams
ExecFuncParams -> ExecExp ExecFuncParamsE
ExecFuncParamsE -> Tokens::Comma ExecFuncParams | Tokens::Null

First(ExecVar) = {Tokens::Identity}
First(ExecVarT) = {Tokens::LeftMB, Tokens::ShouldReturn, Tokens::Null, Tokens::LeftC}
First(ExecVarSoE) = {Tokens::Int, Tokens::Identity}
First(ExecFuncP) = {Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus, Tokens::Null}
First(ExecFuncParams) = {Tokens::Str, Tokens::Int, Tokens::Decimal, Tokens::Bool, Tokens::Match, Tokens::Identity, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus}
First(ExecFuncParamsE) = {Tokens::Comma, Tokens::Null}

Follow(ExecVar) = {Tokens::PlusIs, Tokens::MinusIs, Tokens::DivIS, Tokens::MulIs, Tokens::ModIs, Tokens::Is, Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div, Tokens::Mod, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecVarT) = {Tokens::PlusIs, Tokens::MinusIs, Tokens::DivIS, Tokens::MulIs, Tokens::ModIs, Tokens::Is, Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div, Tokens::Mod, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecVarSoE) = {Tokens::LeftMB, Tokens::ShouldReturn, Tokens::LeftC, Tokens::PlusIs, Tokens::MinusIs, Tokens::DivIS, Tokens::MulIs, Tokens::ModIs, Tokens::Is, Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div, Tokens::Mod, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon}
Follow(ExecFuncP) = {Tokens::RightC}
Follow(ExecFuncParams) = {Tokens::RightC}
Follow(ExecFuncParamsE) = {Tokens::RightC}

类型声明（单独类型，也可以是复合类型（数组、元组））
元组类型允许空（语句的类型）
ExecType -> Tokens::Identity | Tokens::LeftMB ExecType Tokens::EndExp Tokens::Int Tokens::RightMB | Tokens::LeftC ExecTypesP Tokens::RightC
ExecTypesP -> ExecTypesParams | Tokens::Null
ExecTypesParams -> ExecType ExecTypesParamsE
ExecTypesParamsE -> Tokens::Comma ExecTypesParams | Tokens::Null

First(ExecType) = {Tokens::Identity, Tokens::LeftMB, Tokens::LeftC}
First(ExecTypesP) = {Tokens::Identity, Tokens::LeftMB, Tokens::LeftC, Tokens::Null}
First(ExecTypesParams) = {Tokens::Identity, Tokens::LeftMB, Tokens::LeftC}
First(ExecTypesParamsE) = {Tokens::Comma, Tokens::Null}

Follow(ExecType) = {Tokens::Is, Tokens::Comma, Tokens::EndExp, Tokens::RightC, Tokens::LeftBC, Tokens::RightBC}
Follow(ExecTypesP) = {Tokens::RightC}
Follow(ExecTypesParams) = {Tokens::RightC}
Follow(ExecTypesParamsE) = {Tokens::RightC}

*/

/* 函数体

函数体中以语句为基本单位（最后返回值可以是表达式，语法糖）
DefineFn -> Tokens::Fn Tokens::Identity Tokens::LeftC FnP Tokens::RightC FnReturn Tokens::LeftBC FnBody Tokens::RightBC
可以不存在参数
FnP -> Tokens::Null | FnParams
支持复合类型
FnParams -> Tokens::Identity Tokens::Semicolon ExecType FnParamsE
FnParamsE -> Tokens::Comma FnParams | Tokens::Null
可以不存在返回值（默认返回值为元组 ()? ）
FnReturn -> Tokens::Null | Tokens::ShouldReturn ExecType
函数体由语句组成
FnBody -> ExecSentence FnBody | Tokens::Null

First(DefineFn) = {Tokens::Fn}
First(FnP) = {Tokens::Identity, Tokens::Null}
First(FnParams) = {Tokens::Identity}
First(FnParamsE) = {Tokens::Comma, Tokens::Null}
First(FnReturn) = {Tokens::ShouldReturn, Tokens::Null}
First(FnBody) = {Tokens::Let, Tokens::Identity, Tokens::If, Tokens::While, Tokens::LeftBC, Tokens::Return, Tokens::Break}

Follow(DefineFn) = {Tokens::Struct, Tokens::Fn, Tokens::End, Tokens::RightBC}
Follow(FnP) = {Tokens::RightC}
Follow(FnParams) = {Tokens::RightC}
Follow(FnParamsE) = {Tokens::RightC}
Follow(FnReturn) = {Tokens::LeftBC}
Follow(FnBody) = {Tokens::RightBC}

*/

/// 抽象语法树 AST
pub type AST = Tree<ASTNode>;

/// 非终结符
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum NT {
    // 开始符号
    Merilog,
    // 结构体声明
    DefineStruct,
    DefineStructBody,
    DefineStructBodyNext,
    // 变量声明
    ExecStmt,
    DefineVar,
    DefineVarMutable,
    DefineVarS,
    DefineVarType,
    DefineVarValue,
    DefineVarE,
    // 语句
    ExecSentence,
    // 表达式
    ExecExp,
    ExecR1,
    ExecExpAndS,
    ExecR2,
    ExecExpOr,
    ExecR3,
    ExecExpAnd,
    ExecR4,
    ExecExpEq,
    ExecR5,
    ExecExpLGq,
    ExecR6,
    ExecExpAddOp,
    ExecR7,
    ExecExpMultiOp,
    ExecR8,
    ExecExpSigOp,
    ExecExpN,
    Eqs,
    LGqs,
    AddOps,
    MultiOps,
    SigOps,
    Ops,
    // 返回
    ExecRet,
    // Break
    ExecBreak,
    // 赋值语句
    ExecIs,
    ExecIsW,
    // Match 选择
    ExecMatch,
    ExecMatchS,
    ExecMatchE,
    // If 条件判断
    ExecIf,
    ExecIfE,
    ExecIfEi,
    // While 循环
    ExecWhile,
    // Loop 循环
    ExecLoop,
    // 成员引用
    ExecVar,
    ExecVarT,
    ExecVarSoE,
    ExecFuncP,
    ExecFuncParams,
    ExecFuncParamsE,
    // 类型声明
    ExecType,
    ExecTypesP,
    ExecTypesParams,
    ExecTypesParamsE,
    // 函数体
    DefineFn,
    FnP,
    FnParams,
    FnParamsE,
    FnReturn,
    FnBody
}

/// 抽象语法树结点
#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    /// 终结符
    T(Tokens),
    /// 非终结符
    NT(NT)
}

impl Visualize for ASTNode {
    fn visualize(&self) -> String {
        match self {
            ASTNode::T(t) => format!("{:?}", t),
            ASTNode::NT(nt) => format!("{:?}", nt)
        }
    }

    fn emphasize(&self) -> bool {
        match self {
            ASTNode::T(_) => true,
            ASTNode::NT(_) => false
        }
    }
}