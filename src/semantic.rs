//! 语义分析

pub mod llvmir_gen;

/*

语法制导翻译方案，翻译成 LLVM IR，通过调用封装 LLVM IR API 的库（inkwell）生成

（经过化简，是语法分析简化处理过后生成的 AST 的等价文法）

Merilog -> DefineStruct Merilog
Merilog -> DefineFn Merilog
Merilog -> Tokens::Null

DefineStruct -> Tokens::Identity DefineStructBody
DefineStructBody -> Tokens::Identity ExecType DefineStructBodyNext
DefineStructBody -> DefineFn DefineStructBodyNext
DefineStructBodyNext -> DefineStructBody
DefineStructBodyNext -> Tokens::Null

DefineVar -> DefineVarMutable
DefineVarMutable -> Tokens::mut DefineVarS
DefineVarMutable -> DefineVarS
DefineVarS -> Tokens::Identity DefineVarType DefineVarValue DefineVarE
DefineVarType -> Tokens::Null
DefineVarType -> ExecType
DefineVarValue -> Tokens::Null
DefineVarValue -> ExecExp
DefineVarE -> DefineVarS
DefineVarE -> Tokens::Null

ExecSentence -> ExecStmt
ExecSentence -> ExecIs
ExecSentence -> ExecIf
ExecSentence -> ExecWhile
ExecSentence -> ExecLoop
ExecSentence -> ExecRet
ExecSentence -> ExecBreak

ExecExp -> ExecExpAndS R1
R1 -> Tokens::OrS ExecExp R1
R1 -> Tokens::Null
ExecExpAndS -> ExecExpOr R2
R2 -> Tokens::AndS ExecExpAndS R2
R2 -> Tokens::Null
ExecExpOr -> ExecExpAnd R3
R3 -> Tokens::Or ExpecExpOr R3
R3 -> Tokens::Null
ExecExpAnd -> ExecExpEq R4
R4 -> Tokens::And ExecExpAnd R4
R4 -> Tokens::Null
ExecExpEq -> ExecExpLGq R5
R5 -> Eqs ExecExpEq R5
R5 -> Tokens::Null
ExecExpLGq -> ExecExpAddOp R6
R6 -> LGqs ExecExpLGq R6
R6 -y
> Tokens::Null
ExecExpAddOp -> ExecExpMultiOp R7
R7 -> AddOps ExecExpAddOp R7
R7 -> Tokens::Null
ExecExpMultiOp -> ExecExpSigOp R8
R8 -> MultiOps ExecExpMultiOp R8
R8 -> Tokens::Null
ExecExpSigOp -> SigOps ExecExpN
ExecExpN -> Ops
ExecExpN -> Tokens::LeftC ExecExp Tokens::RightC
Eqs -> Tokens::Eq
Eqs -> Tokens::Ne
LGqs -> Tokens::Gt
LGqs -> Tokens::Lt
LGqs -> Tokens::Ge
LGqs -> Tokens::Le
AddOps -> Tokens::Plus
AddOps -> Tokens::Minus
MultiOps -> Tokens::Mul
MultiOps -> Tokens::Div
MultiOps -> Tokens::Mod
SigOps -> Tokens::Negate
SigOps -> Tokens::Plus
SigOps -> Tokens::Minus
SigOps -> Tokens::Null
Ops -> Tokens::Str
Ops -> Tokens::Int
Ops -> Tokens::Decimal
Ops -> Tokens::Bool
Ops -> ExecMatch
Ops -> ExecVar

ExecStmt -> DefineVar

ExecRet -> ExecExp

ExecBreak -> Tokens::Break

ExecIS -> ExecVar ExecIsW ExecExp
ExecIsW -> Tokens::PlusIs
ExecIsW -> Tokens::MinusIs
ExecIsW -> Tokens::DivIs
ExecIsW -> Tokens::MulIs
ExecIsW -> Tokens::ModIs
ExecIsW -> Tokens::Is

ExecMatch -> ExecExp ExecMatchS
ExecMatchS -> ExecExp FnBody ExecMatchE
ExecMatchE -> ExecMatchS
ExecMatchE -> Tokens::Null

ExecIf -> ExecExp FnBody ExecIfE
ExecIfE -> Tokens::Else ExecIfEi FnBody ExecIfE
ExecIfE -> Tokens::Null
ExecIfEi -> Tokens::If ExecExp
ExecIfEi -> Tokens::Null

ExecWhile -> ExecExp FnBody

ExecLoop -> FnBody ExecExp

- 成员引用，可以是函数调用

ExecVar -> Tokens::Identity ExecVarT
- - 数组引用
ExecVarT -> Tokens::LeftMB Tokens::Int Tokens::RightMB ExecVarT
- - 元组、结构体引用
ExecVarT -> Tokens::ShouldReturn ExecVarSoE ExecVarT
- - 直接变量引用
ExecVarT -> Tokens::Null
- - 调用函数
ExecVarT -> Tokens::LeftC ExecFuncP Tokens::RightC ExecVarT
- - 元组、结构体引用
ExecVarSoE -> Tokens::Int
ExecVarSoE -> Tokens::Identity
- - 获取函数调用参数
ExecFuncP -> Tokens::Null
ExecFuncP -> ExecFuncParams
ExecFuncParams -> ExecExp ExecFuncParamsE
ExecFuncParamsE -> ExecFuncParams
ExecFuncParamsE -> Tokens::Null

- 单独类型（基本类型 + 声明的新类型）
ExecType -> Tokens::Identity
- 数组
ExecType -> Tokens::LeftMB ExecType Tokens::EndExp Tokens::Int Tokens::RightMB
- 处理元组
ExecType -> Tokens::LeftC ExecTypesP Tokens::RightC
- 允许空元组
ExecTypesP -> ExecTypesParams
ExecTypesP -> Tokens::Null
- 元组内的参数
ExecTypesParams -> ExecType ExecTypesParamsE
ExecTypesParamsE -> ExecTypesParams
ExecTypesParamsE -> Tokens:Null

DefineFn -> Tokens::Identity FnP FnReturn {生成一个函数，包括函数名，函数参数列表（包括行参名称，参数类型），函数返回类型，这些可以用 inkwell 的 AnyValueEnum 枚举保存} FnBody
FnP -> Tokens::Null | FnParams
FnParams -> Tokens::Identity ExecType FnParamsE
FnParamsE -> FnParams | Tokens::Null
FnReturn -> Tokens::Null | ExecType
FnBody -> ExecSentence FnBody | Tokens::Null

 */
