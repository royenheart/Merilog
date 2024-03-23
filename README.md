# 兰州大学编译原理实验

Rust + LLVM-IR（inkwell LLVM15-0）

感谢 [sbwtw](https://github.com/sbwtw/MyParser) 的设计思路

## Usage

```bash
Merilog use method: [-h] [-v] [ -P | -Preprocess ] [ -L | --Lex ] [ -S | -Syntax [--Visual <vi_output>]] [-I | --IR] [-o <output>] <input>
```

## 功能

将代码翻译成 LLVM IR，可通过 JIT 编译执行查看运行结果。

### 源语⾔定义

1. 基于表达式，任何语句都具有返回值，语句（如 loop 、 while 、变量声明等）默认返回空元组 `()` 。没有 void type ，使⽤空元组替代。
2. ⽀持结构体声明和创建、⽀持结构体内函数声明，可通过具体的结构体实例调⽤函数。
3. ⽀持函数调⽤，函数可定义函数参数、函数返回类型以及函数体。禁⽌空函数体，函数默认返回空元组（即不具名的⽆字段的结构体），函数参数可为空，函数返回类型可为空。结构体内函数使⽤ self 关键字进⾏⾃⾝的调⽤，函数⽀持递归调⽤，且可调⽤⾃⾝。
4. 源语⾔的函数调⽤目前仅实现了传值、不⽀持传引⽤（指针），且传⼊的值为不可变，⽆法直接修改传⼊的值。
5. 源语⾔⽀持复杂表达式的解析，基本类型包括 i32,f32,bool,str （后期可以拓展⾄多种类型），各类型的计算目前暂不⽀持⾃动类型转换。 gen_op 函数为具体的 lhs 和 rhs 运算实现函数，原理上可以拓展⾄各种类型之间的相互运算。
6. ⽀持复杂控制语句以及嵌套使⽤，⽬前⽀持的控制语句包括： while 、 loop 、 if 、 match ，其中 match 只能⽤在表达式中，其控制流程类似 switch-case ，⽀持嵌套使⽤。
7. ⽀持 return 语句，在 match 中， return 实际传递给表达式右值，即表达式中， match 本⾝会返回⼀个右值供计算，其他控制语句以及函数体的 return 表⽰函数返回。
8. ⽀持 break 语句，仅限在 while 、 loop 循环控制语句中使⽤，表⽰跳出当前循环。
9. 变量声明⽀持可变或不可变声明（ mut ），默认为不可变，即声明后，变量将⽆法进⾏更改；变量声明可声明变量类型和变量被赋予的值，变量类型可为空，此时将根据赋值的值进⾏类型推导，变量赋值可为空，此时变量将被赋予默认值，即该类型的默认值。⽬前⽆法进⾏延迟初始化（即类型和值都未被声明），需要后⾯进⾏⼀定的修改。
10. ⽀持数组、空元组（不具名的结构体）的复合类型的声明以及对应变量的引⽤。但由于⽂法设计在左右值、表达式上存在⼀定的问题，⽬前⽆法实现数组、元组的初始化。

### 编译器目前支持的功能

1. ⽀持命名空间和作⽤域概念，⽀持隐藏机制。命名空间将类型（结构体类型，即⾃定义类型）和具体值（变量、函数等）的声明分离。对变量、函数进⾏引⽤（调⽤）时，将从当前作⽤域开始，往上遍历祖先节点（作⽤域），将找到的第⼀个对应符号作为引⽤（调⽤）对象。当同⼀作⽤域声明了多个同名函数、变量时，将执⾏隐藏机制，即之前被声明的同名对象将被隐藏，被查找时将返回最后被声明的对象。

    ![](https://royenheart.com/static/img/190b60073812420dded783086f631612.symbol-table.webp)

2. ⽀持类型检查，类型检查在语义分析过程中进⾏，包括我⾃⼰实现的类型检查 + LLVM 提供的对 LLVM IR 代码的类型检查。我⾃⼰实现的类型检查包括函数返回值和函数返回类型的⽐较、算数表达式中 lhs 和 rhs 的类型⽐较等、分⽀类型检查等。其余类型检查可交由 LLVM 提供的对 IR 代码的静态检查。
3. ⽀持核⼼库加载，原理为在分析之前在符号表中加上库的函数、对象、类型等，⽬前测试仅先加载 puts 函数，该函数将在链接阶段链上系统的 puts 函数，从⽽实现打印输出。

### 效果

通过加载核心库（伪）打印输出 "Hello World"

![](https://royenheart.com/static/img/dc2dc49668dc0016e74ea80e7ae19a48.merilog.webp)

## 文法

```rust
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
```

仿照 rust 的部分语法，文法由于设计时间较短，存在很多问题，目前比较抽象，同时存在一定的 BUG。

## 目录结构

`examples` 目录下存放了使用案例以及源语言的测试用例。