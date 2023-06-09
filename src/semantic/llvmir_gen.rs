//! LLVM IR Generate

// 存在基本类型、常量
// 2023.6.3，_ 和 self 成为关键词可能好一点。
// 特判：_ _() () self
// 2023.6.3，赋值时出错了。。，赋值时右值使用了表达式 ExecExp，ExecExp 受 ExecVar 成员引用影响，无法定义元组，无法构造结构体，无法构造数组。。。
// () 只能在 ExecType 中出现，identity 不可能出现，ExecExp 和 ExecVar 不可能出现
// () 是空元组基本类型，被识别成 struct_type([])，即空的不具名结构体
// _() 是空元组常量，被识别成 struct_type([]).const...，空的不具名结构体的 const 常量
// _ 也被视作一个常量，其引用和值都是 int(0) / self.context.i32_type().const_zero()，但是不是一个正常值，一些运算无法直接参与
// ×不可行：_(3, 3) 等构造一个具有变量的元组量，文法已经缺少这块了。
// 之后需要文法支持元组定义、结构体定义才行。ExecVar 当初考虑成员引用只考虑了引用变量，没想到右值（字面量）比如 (3, 4) 这种也可以引用: (3, 4).0 为 3，只是由于没有值存储，无法对字面量进行赋值。
// 目前解决方法：结构体在结构体定义时声明一个同名函数可以创建结构体（可实现，结构体类型和具体值是分离的，可以添加至上层作用域），元组和数组也可通过实现声明函数？（难以实现，因为有很多类型，又可以有很多组合方式。。。）
// 函数的默认返回值都是空元组常量，即空的不具名结构体的 const 常量
// self 在结构体函数中被强行传入符号，之后当作正常符号处理，只是无法被声明（即无法被隐藏）
// 2023.6.3，ExecType 倒是可以直接定义一个空元组类型 ()
// 2023.6.4，语言不支持多模块、包、全局变量声明（静态变量），全局变量声明只包括
// 2023.6.5，符号表的引用还是有点问题，结构体若内部字段按顺序类型一致，别名添加的时候会识别成最后添加的
// struct a {  |  struct b {
//     a: i32, |      c: i32,
//     b: u32  |      d: u32,
// }           |  }
// 这两个结构体能注册，但判定类型的时候会识别成同一个。。。。，因为是不具名结构体。。。
// 仔细研究下 IR 的数据类型和源语言的数据类型，之后把符号表改好点。。
// 2023.6.5，match 表达式，由于需要事先知道各个块的类型才能分析每个块中的函数（需要对已经写好的基本块进行修改、回填），太复杂就先直接判定类型为 match 判断的值的类型
// 之后再好好设计下
// 2023.6.5，应该从文法层面就区分左值右值这种，全用 ExecExp 和 ExecVar 有点难绷（这两者设计地不是很好区分）
// 需要插入代码的基本块每次由对应函数负责进行跳转
// 支持 break 跳转的语句：
// while，loop，if
// match 不提供 break 跳转
// 由于基本类型判断还缺少很多类型，bool 的 i1type 等需要转换成能识别的（i1 type 必须转换成 i32 type 才能继续使用），需要继续完善
// gen_op 可以有很多种情况，迫于时间问题就不一一实现了（做得到）

use std::{
    path::Path, collections::HashMap, borrow::Borrow,
};

use id_tree::NodeId;
use id_tree_layout::Visualize;
use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType, AnyType},
    values::{
        AnyValue, AnyValueEnum, BasicValue, BasicValueEnum, FunctionValue, InstructionValue, PointerValue, StructValue, BasicMetadataValueEnum,
    },
    AddressSpace, OptimizationLevel,
};
use lazy_static::lazy_static;

use crate::{
    lex::Tokens,
    syntax::{ASTNode, AST, NT},
    table::symbol::SymbolManager,
};

/// 过滤特定节点 \
/// 以后序遍历，跳过（不包括本身）特定节点下的子树
fn post_traversal_retain_and_skipsubtree(
    tree: &AST,
    root: &NodeId,
    retain_func: fn(&ASTNode) -> bool,
    skip_func: fn(&ASTNode) -> bool,
) -> Vec<NodeId> {
    let node = tree.get(root).unwrap();
    let mut ret = vec![];
    let inner_data = node.data();

    if !skip_func(inner_data) {
        for child in node.children().iter() {
            let mut c = post_traversal_retain_and_skipsubtree(tree, child, retain_func, skip_func);
            ret.append(&mut c);
        }
    }
    ret.push(root.clone());
    ret.into_iter().filter(|x| {
        let inner_data = tree.get(x).unwrap().data();
        !retain_func(inner_data)
    }).collect()
}

fn into_basic_value(any_value: AnyValueEnum) -> Result<BasicValueEnum, String> {
    match any_value {
        AnyValueEnum::ArrayValue(v) => Ok(v.into()),
        AnyValueEnum::IntValue(v) => Ok(v.into()),
        AnyValueEnum::FloatValue(v) => Ok(v.into()),
        AnyValueEnum::PointerValue(v) => Ok(v.into()),
        AnyValueEnum::StructValue(v) => Ok(v.into()),
        AnyValueEnum::VectorValue(v) => Ok(v.into()),
        _ => Err(format!("无法转换 AnyValueEnum 至 BasicValueEnum：{:?}", any_value)),
    }
}

fn into_basic_type(any_type: AnyTypeEnum) -> Result<BasicTypeEnum, String> {
    match any_type {
        AnyTypeEnum::ArrayType(v) => Ok(v.into()),
        AnyTypeEnum::IntType(v) => Ok(v.into()),
        AnyTypeEnum::FloatType(v) => Ok(v.into()),
        AnyTypeEnum::PointerType(v) => Ok(v.into()),
        AnyTypeEnum::StructType(v) => Ok(v.into()),
        AnyTypeEnum::VectorType(v) => Ok(v.into()),
        _ => Err(format!("无法转换 AnyTypeEnum 至 BasicTypeEnum：{:?}", any_type)),
    }
}

fn into_basic_metadata_type(any_type: AnyTypeEnum) -> Result<BasicMetadataTypeEnum, String> {
    match any_type {
        AnyTypeEnum::ArrayType(v) => Ok(v.into()),
        AnyTypeEnum::IntType(v) => Ok(v.into()),
        AnyTypeEnum::FloatType(v) => Ok(v.into()),
        AnyTypeEnum::PointerType(v) => Ok(v.into()),
        AnyTypeEnum::StructType(v) => Ok(v.into()),
        AnyTypeEnum::VectorType(v) => Ok(v.into()),
        _ => Err(format!("无法转换 AnyTypeEnum 至 BasicMetadataTypeEnum：{:?}", any_type)),
    }
}

fn into_basic_metadata_value(any_value: AnyValueEnum) -> Result<BasicMetadataValueEnum, String> {
    match any_value {
        AnyValueEnum::ArrayValue(v) => Ok(v.into()),
        AnyValueEnum::IntValue(v) => Ok(v.into()),
        AnyValueEnum::FloatValue(v) => Ok(v.into()),
        AnyValueEnum::PointerValue(v) => Ok(v.into()),
        AnyValueEnum::StructValue(v) => Ok(v.into()),
        AnyValueEnum::VectorValue(v) => Ok(v.into()),
        _ => Err(format!("无法转换 AnyValueEnum 至 BasicMetadataValueEnum：{:?}", any_value)),
    }
}

#[inline]
fn same_type(me: &AnyTypeEnum, other: &AnyTypeEnum) -> bool {
    me == other
}

// 如何处理单目运算的优先级？
// 遇到可能的单目运算，检测下一个符号是否是 ExecExpN
// 即是否是 Tokens::Str Tokens::Int Tokens::Decimal Tokens::Bool ExecMatach ExecVar ExecExp
// (忽略括号)
// + 单目其实是 + +v，- 单目其实是 + -v，而 ! 单目直接进行运算
lazy_static!{
    static ref TOK_PRIORITIES: HashMap<String, u8> = {
        let mut r = HashMap::new();
        r.insert(ASTNode::T(Tokens::LeftC).visualize(), 1);
        r.insert(ASTNode::T(Tokens::RightC).visualize(), 1);
        r.insert(ASTNode::T(Tokens::Negate).visualize(), 2);
        r.insert(ASTNode::T(Tokens::Mul).visualize(), 3);
        r.insert(ASTNode::T(Tokens::Div).visualize(), 3);
        r.insert(ASTNode::T(Tokens::Mod).visualize(), 3);
        r.insert(ASTNode::T(Tokens::Plus).visualize(), 4);
        r.insert(ASTNode::T(Tokens::Minus).visualize(), 4);
        r.insert(ASTNode::T(Tokens::Gt).visualize(), 5);
        r.insert(ASTNode::T(Tokens::Ge).visualize(), 5);
        r.insert(ASTNode::T(Tokens::Lt).visualize(), 5);
        r.insert(ASTNode::T(Tokens::Le).visualize(), 5);
        r.insert(ASTNode::T(Tokens::Eq).visualize(), 6);
        r.insert(ASTNode::T(Tokens::Ne).visualize(), 6);
        r.insert(ASTNode::T(Tokens::And).visualize(), 7);
        r.insert(ASTNode::T(Tokens::Or).visualize(), 8);
        r.insert(ASTNode::T(Tokens::AndS).visualize(), 9);
        r.insert(ASTNode::T(Tokens::OrS).visualize(), 10);
        r
    };
}

#[inline]
fn judge_left_gt_right_priority(left: &ASTNode, right: &ASTNode) -> bool {
    (*TOK_PRIORITIES).get(&left.visualize()) > (*TOK_PRIORITIES).get(&right.visualize())
}

/// has_type 为 Type，表示类型（对于变量来说，需要指示其类型，Value 不一定代表（可能是 PointerValue））\
/// has_type 为 None，则表示为未确定类型，需要延迟初始化，需要传入需要回填的基本块的ID
#[derive(Debug, Clone)]
pub enum HasType<'a> {
    Type(AnyTypeEnum<'a>),
    None(InstructionValue<'a>)
}

#[derive(Debug)]
pub struct VType<'a> {
    value: Option<AnyValueEnum<'a>>,
    has_env: Option<NodeId>,
    has_type: HasType<'a>,
    is_mut: bool
}

impl<'a> VType<'a> {
    fn new(value: Option<AnyValueEnum<'a>>, has_env: Option<NodeId>, has_type: HasType<'a>, is_mut: bool) -> Self {
        Self { value, has_env, has_type, is_mut }
    }
}

/// 类型内情量，当定义了新类型（如结构体）且需要存储额外控制信息时，需要到此查询
pub struct TyType<'a> {
    type_value: AnyTypeEnum<'a>,
    params: Vec<&'a str>,
    has_env: Option<NodeId>
}

impl<'a> TyType<'a> {
    fn new(type_value: AnyTypeEnum<'a>, params: Vec<&'a str>, has_env: Option<NodeId>) -> Self {
        Self { type_value, params, has_env }
    }
}

pub struct IrGen<'a, 'ctx> {
    ast: &'ctx AST,
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    /// 类型符号表： \
    /// 值符号表：AnyValueEnum 表示具体值（可进行反填），NodeId 表示值下所属的作用域，str 为类型（可进行反填），bool 表示是否可变（true 为可变，false 为不可变）
    symbols: &'ctx mut SymbolManager<TyType<'a>, VType<'a>>
}

impl<'a, 'ctx> IrGen<'a, 'ctx> where 'ctx: 'a {
    pub fn new(
        ast: &'ctx AST,
        context: &'ctx Context,
        symbols: &'ctx mut SymbolManager<TyType<'a>, VType<'a>>,
        module_name: &str
    ) -> IrGen<'a, 'ctx> {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        IrGen {
            ast,
            context,
            module,
            builder,
            symbols
        }
    }

    // 导出生成的 LLVM IR 代码
    pub fn dump(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn dump_to_file<T: AsRef<Path>>(&self, file: T) -> Result<(), String> {
        let f = file.as_ref();
        match self.module.print_to_file(f) {
            Ok(_) => Ok(()),
            Err(x) => Err(x.to_string()),
        }
    }

    pub fn jit_engine(
        &self,
        optimization_level: OptimizationLevel,
    ) -> Result<ExecutionEngine, String> {
        match self.module.create_jit_execution_engine(optimization_level) {
            Ok(x) => Ok(x),
            Err(x) => Err(x.to_string()),
        }
    }

    pub fn jit_execute<RetT>(
        &self,
        optimization_level: OptimizationLevel,
        func_name: &str,
    ) -> Result<RetT, String> {
        match (self.module.verify(), self.jit_engine(optimization_level)) {
            (Ok(_), Ok(engine)) => {
                match unsafe {engine.get_function::<unsafe extern "C" fn() -> RetT>(func_name)} {
                    Ok(f) => unsafe {
                        Ok(f.call())
                    },
                    Err(e) => {
                        Err(e.to_string())
                    }
                }
            }
            (Err(x), _) => Err(format!("Static Analysis Failed: {}", x)),
            (_, Err(x)) => Err(format!("JIT Engine Create Faild: {}", x)),
        }
    }

    /// 加载核心库（伪）
    fn load_core(&mut self, env: &NodeId) {
        let i32_type = self.context.i32_type();
        let str_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let printf_type = i32_type.fn_type(&[str_type.into()], true);
        let printf_func = self.module.add_function("puts", printf_type, Some(inkwell::module::Linkage::External));
        let printf_func_v = printf_func.as_any_value_enum();
        self.push_identi_values("puts", VType::new(Some(printf_func_v), None, HasType::Type(printf_type.as_any_type_enum()), false), env);
    }

    /// Generate LLVM IR Code
    pub fn gen(&mut self) -> Result<(), String> {
        let ids = self.children_ids(&self.ast.root_node_id().unwrap().clone());
        let root_env = self.get_root_env();
        self.load_core(&root_env);
        for id in ids {
            self.dispatch_node(&id, &root_env)?;
        }

        Ok(())
    }

    // 验证构建的 LLVM IR 代码合法性
    pub fn verify(&self) -> Result<(), String> {
        match self.module.verify() {
            Ok(_) => Ok(()),
            Err(x) => Err(x.to_string())
        }
    }

    /// 子任务分发
    /// 文法开始符号的直接子节点只能是：DefineStruct / DefineFn
    fn dispatch_node(&mut self, root: &NodeId, env: &NodeId) -> Result<(), String> {
        match self.node_data(root) {
            // 函数定义
            ASTNode::NT(NT::DefineFn) => {
                self.def_func(root, env, None)?;
            }
            // 结构体定义
            ASTNode::NT(NT::DefineStruct) => {
                self.def_struct(root, env)?;
            }
            other => {
                unreachable!("试图定义：{:?} 源代码只允许定义结构体和函数", other)
            }
        };
        Ok(())
    }

    /// 在当前作用域中添加函数符号，并新建函数的作用域，DefineFn
    /// 此时不注册新类型，但注册变量（相当于引用，函数指针，类型为 FunctionValue），自己本身就是一个特殊实例
    fn def_func(
        &mut self,
        root: &NodeId,
        env: &NodeId,
        in_struct: Option<AnyTypeEnum<'ctx>>,
    ) -> Result<AnyValueEnum<'ctx>, String> {
        let ids = self.children_ids(root);
        let fn_name = match self.node_data(&ids[0]) {
            ASTNode::T(Tokens::Identity(x)) => x,
            other => {
                unreachable!(
                    "函数定义应以（AST中）标识符开头，出现: {:?}，函数定义语法分析出错",
                    other
                )
            }
        };
        let mut args_type = vec![];
        let mut args_meta_type = vec![];
        let mut args_name = vec![];
        let mut func_ret = AnyTypeEnum::StructType(self.get_void_type());
        // 创建新的作用域，该作用域用于其函数体，其他同级符号无法直接访问
        let new_env = self.symbols.create_env(env);

        // 当前作用域值符号表不能出现同名函数
        if self.identi_looknow_values(fn_name, env).is_some() {
            return Err(format!("已存在函数定义：{}", fn_name));
        }

        for id in ids.into_iter().skip(1) {
            let what_symbol = self.node_data(&id);
            match what_symbol {
                // 检查函数参数，遍历 FnP 为头节点的子树，将其中的 Tokens::Identity（形参名称）和 ExecType（形参类型）检索出来
                ASTNode::NT(NT::FnP) => {
                    let params_nodes = post_traversal_retain_and_skipsubtree(
                        self.ast,
                        &id,
                        |x| !matches!(x, ASTNode::T(Tokens::Identity(_)) | ASTNode::NT(NT::ExecType)),
                        |x| matches!(x, ASTNode::NT(NT::ExecType)),
                    );
                    let mut itr = params_nodes.into_iter();
                    while let Some(x) = itr.next() {
                        if let ASTNode::T(Tokens::Identity(argname)) = self.node_data(&x) {
                            let t = self.resolv_exec_type(&itr.next().unwrap(), env)?;
                            args_meta_type.push(into_basic_metadata_type(t).unwrap());
                            args_type.push(t);
                            args_name.push(argname.as_str());
                        }
                    }
                }
                // 检查函数返回类型
                ASTNode::NT(NT::FnReturn) => {
                    let exect = self.children_ids(&id);
                    func_ret = self.resolv_exec_type(&exect[0], env)?;
                }
                // 检查是否是 FnBody，是，则分析 FnBody，此时先进行函数注册
                ASTNode::NT(NT::FnBody) => {
                    // 如果是在结构体中，需要额外添加一个参数
                    if let Some(stype) = in_struct {
                        args_type.push(stype);
                        args_name.push("self");
                        args_meta_type.push(into_basic_metadata_type(stype).unwrap())
                    }
                    // 建立函数，此时判断函数参数、函数返回类型是否为空，否则默认为空元组
                    let func_type = into_basic_type(func_ret)
                        .unwrap()
                        .fn_type(&args_meta_type, false);
                    let func_v = self.module.add_function(fn_name.as_str(), func_type, None);
                    // 将函数自身加入符号表，类型就是函数类型本身
                    self.push_identi_values(fn_name, VType::new(Some(func_v.as_any_value_enum()), Some(new_env.clone()), HasType::Type(func_ret), false), env);
                    // 对函数创建新的基本块
                    let basicb = self.context.append_basic_block(func_v, "func_basic_b");
                    // 跳转至新函数体的基本块进行语句的构建
                    self.builder.position_at_end(basicb);
                    // 2023.6.6，使用传值进行函数调用，还不支持传指针。
                    // 2023.6.6，由于使用传值，而又要构造符号，因此需要在函数开头构造语句，构建对应的符号（指针），存储传入的值
                    for (index, va) in func_v.get_param_iter().enumerate() {
                        let va = va;
                        let ps = self.builder.build_alloca(into_basic_type(args_type[index]).unwrap(), "load_func_p");
                        self.builder.build_store(ps, va);
                        self.push_identi_values(
                            args_name[index],
                            // 2023.6.4，默认参数不可变，由于文法不能在函数定义时指定类型为可变
                            VType::new(Some(ps.as_any_value_enum()), None, HasType::Type(args_type[index]), false),
                            &new_env,
                        );
                    }
                    // 开始在新的作用域和第一个基本块下分析函数体 FnBody，还不用考虑跳转块
                    let need_default_ret = func_ret == AnyTypeEnum::StructType(self.get_void_type());
                    let (r, _) = self.resolv_fn_body(&id, &new_env, None, &func_v, None, need_default_ret)?;
                    // FnBody 分析完毕，返回其返回类型，检查类型是否与函数返回类型一致
                    match same_type(&func_ret, &r) {
                        true => (),
                        false => {
                            return Err(format!("函数返回类型: {:?} 和函数体返回类型: {:?} 不一致", func_ret, r));
                        }
                    }

                    return Ok(func_v.as_any_value_enum());
                }
                other => {
                    unreachable!("函数体定义遇见不该出现的符号：{:?}，函数定义错误", other)
                }
            }
        }

        Err(format!("禁止空函数"))
    }

    /// 在当前作用域中定义结构体，DefineStruct
    /// 注册一个新结构体类型（不是变量）
    /// 结构体下属需要放入变量名称以便进行类型判断（因为不是具体的值，还不用完全确定）
    /// 结构体下属的函数体需要实现确定
    fn def_struct(&mut self, root: &NodeId, env: &NodeId) -> Result<AnyTypeEnum<'ctx>, String> {
        let ids = self.children_ids(root);
        let struct_name = match self.node_data(&ids[0]) {
            ASTNode::T(Tokens::Identity(x)) => x,
            other => {
                unreachable!(
                    "结构体定义应以（AST中）标识符开始，初始为符号：{:?}，结构体定义的语法分析出错",
                    other
                )
            }
        };
        let mut args_type = vec![];
        let mut args_type_func = vec![];
        let mut funcs = vec![];
        // 首先创建一个作用域，专属于结构体，结构体函数、字段只有在结构体作用域下才能访问
        // 引用结构体成员相当于进入结构体作用域
        let new_env = self.symbols.create_env(env);

        // 当前作用域类型符号表不能出现同名结构体
        if self.identi_looknow_types(struct_name, env).is_some() {
            return Err(format!("已存在结构体类型：{}", struct_name));
        }

        // 获取结构体字段（包括变量和函数，函数用 DefineFn 处理，变量遇到 Tokens::Identity 加上之后的 ExecType 进行处理）
        let nodes = post_traversal_retain_and_skipsubtree(
            self.ast,
            &ids[1],
            |x| {
                !matches!(x,
                    ASTNode::T(Tokens::Identity(_)) | 
                    ASTNode::NT(NT::ExecType) | 
                    ASTNode::NT(NT::DefineFn)
                )
            },
            |x| {
                matches!(x,
                    ASTNode::T(Tokens::Identity(_)) | 
                    ASTNode::NT(NT::ExecType) | 
                    ASTNode::NT(NT::DefineFn)
                )
            },
        );
        let mut itr = nodes.into_iter();
        let mut identis = vec![];
        while let Some(n) = itr.next() {
            match self.node_data(&n) {
                // 结构体字段，创建内情向量
                ASTNode::T(Tokens::Identity(identi)) => {
                    let t = self.resolv_exec_type(&itr.next().unwrap(), &new_env)?;
                    match identis.contains(&identi.as_str()) {
                        true => {
                            return Err("结构体内不能同时定义相同的多个字段".to_string());
                        },
                        false => {
                            // 下标就代表偏移量
                            identis.push(identi.as_str());
                        }
                    }
                    args_type.push(into_basic_type(t).unwrap());
                    args_type_func.push(into_basic_metadata_type(t).unwrap());
                }
                // 结构体函数，需要等全部字段声明完毕后再进行
                ASTNode::NT(NT::DefineFn) => {
                    funcs.push(n);
                }
                _ => {
                    unreachable!("结构体定义的语法分析出错")
                }
            }
        }

        // 字段已全部声明完成，在上一层类型作用域注册结构体（需要内情向量，包括含有的字段，下属作用域的引用）
        let struct_params = args_type.as_slice();
        let struct_type = self.context.struct_type(struct_params, false);
        let struct_t_n = struct_type.to_string();
        self.push_identi_type(
            struct_name,
            TyType::new(AnyTypeEnum::StructType(struct_type), identis, Some(new_env.clone())),
            env,
        );
        // 创建别名
        self.symbols.types_add_alias(&struct_t_n, struct_name, env);
        // 添加构造函数，与结构体同名即可，返回是结构体类型，参数为结构体所需的各个字段。
        // 2023.6.6，默认构造传值函数而不是传指针。
        let constructor_params: &[BasicMetadataTypeEnum] = args_type_func.as_slice();
        let constructor_type = struct_type.fn_type(constructor_params, false);
        let constructor = self.module.add_function(struct_name.as_str(), constructor_type, None);
        let constructor_b = self.context.append_basic_block(constructor, struct_name.as_str());
        self.builder.position_at_end(constructor_b);
        let struct_init_ptr = self.builder.build_alloca(struct_type, "init_struct_value");
        for (i, p) in constructor.get_param_iter().enumerate() {
            let param_ptr = unsafe { self.builder.build_gep(p.get_type(), struct_init_ptr, &[self.context.i32_type().const_int(i as u64, false)], "ptr_struct_v") };
            self.builder.build_store(param_ptr, p);
        }
        let struct_init_value = self.builder.build_load(struct_type, struct_init_ptr, "load_struct_value");
        self.builder.build_return(Some(&struct_init_value));
        // 2023.6.7，将构造函数加入 env 中，不添加至 new_env 了
        self.push_identi_values(struct_name, VType::new(Some(constructor.as_any_value_enum()), None, HasType::Type(AnyTypeEnum::StructType(struct_type)), false), env);
        // 接下来解析结构体其他函数，加入 new_env 中
        for func in funcs {
            // 在结构体的作用域内解析函数
            self.def_func(&func, &new_env, Some(AnyTypeEnum::StructType(struct_type)))?;
        }

        // 返回该类型
        return Ok(AnyTypeEnum::StructType(struct_type));
    }

    /// 在当前作用域中定义变量，DefineVar/ExecStmt
    /// 2023.6.6，先取消实现延迟初始化
    /// 变量可能不显式说明类型，此时分两种情况
    /// 1. 后面带赋值表达式，通过计算表达式得到类型，随后设置该变量类型
    /// 2. 延迟初始化，此时需要计算之后的表达式得到类型，然后重新设置该变量类型。即之后的第一个赋值表达式 ExecIs
    /// 3. 当未被初始化被引用到时，无法进行下去。（ExecVar），显示变量未被初始化，不能被使用
    /// 当显式说明类型，以及确定好类型，并且未被隐藏，需要判断：（此时变量类型已经可以被确定）
    /// 1. 定义中被赋值，需要判断左右类型是否相等
    /// 2. 之后被赋值，需要判断左右类型是否相等 ExecIs
    /// 同时还需要传递出可变不可变的方式
    /// 此时不注册新类型，但是要注册新的变量
    /// 类型判断可以通过 LLVM IR 进行检测，不用自行检测
    /// 需要特判 _ 和 self，self 不能直接赋值，左部若为 _，只解析右边的 ExecExp 即可。由于左部是 Identity，所以不可能遇到 _() 这种情况
    fn def_var(&mut self, root: &NodeId, env: &NodeId, func: &FunctionValue) -> Result<AnyTypeEnum<'ctx>, String> {
        // ExecStmt - DefineVar
        let nodes = post_traversal_retain_and_skipsubtree(
            self.ast,
            root,
            |x| {
                !matches!(x,
                    ASTNode::T(Tokens::Mut) | ASTNode::T(Tokens::Identity(_)) | 
                    ASTNode::NT(NT::ExecType) | ASTNode::NT(NT::ExecExp)
                )
            },
            |x| matches!(x, ASTNode::NT(NT::ExecType) | ASTNode::NT(NT::ExecExp)),
        );
        let mut identis = vec![];
        let mut iden_index = -1;
        // 默认不可变
        let mut is_mutable = false;
        for n in nodes {
            match self.node_data(&n) {
                ASTNode::T(Tokens::Mut) => {
                    is_mutable = true;
                },
                ASTNode::T(Tokens::Identity(identi)) if identi.eq("self") => {
                    return Err(format!("self 为特殊保留符号，不能被重新定义"));
                }
                ASTNode::T(Tokens::Identity(identi)) if identi.eq("_") => {
                    identis.push((None, None, None));
                }
                ASTNode::T(Tokens::Identity(identi)) => {
                    identis.push((Some(identi), None, None));
                    iden_index += 1;
                }
                ASTNode::NT(NT::ExecType) => {
                    let t = self.resolv_exec_type(&n, env)?;
                    identis[iden_index as usize].1 = Some(t);
                },
                ASTNode::NT(NT::ExecExp) => {
                    let e = self.resolv_exec_exp(&n, env, func)?;
                    identis[iden_index as usize].2 = Some(e);
                },
                other => {
                    unreachable!("不可能变量声明，遇到符号：{:?}", other)
                }
            }
        }

        for i in identis {
            match i {
                (Some(name), None, None) => {
                    // 2023.6.5 目前先搁置实现，这块从文法上可能就对实现有影响
                    return Err(format!("暂不支持延迟初始化 {:?}", name));
                    let late_init_b = self.context.append_basic_block(*func, "late_init_block");
                    let next_b = self.context.append_basic_block(*func, "next_b");
                    self.builder.build_unconditional_branch(late_init_b);
                    self.builder.position_at_end(late_init_b);
                    let instr = self.builder.build_unconditional_branch(next_b);
                    self.builder.position_at_end(next_b);
                    // 先加入符号表，后续再进行修改
                    self.push_identi_values(name, VType::new(None, None, HasType::None(instr), is_mutable), env);
                },
                (Some(name), None, Some(exp)) => {
                    let t = exp.get_type();
                    let v = self.builder.build_alloca(into_basic_type(t).unwrap(), name.as_str());
                    self.builder.build_store(v, into_basic_value(exp).unwrap());
                    self.push_identi_values(name, VType::new(Some(v.as_any_value_enum()), None, HasType::Type(t), is_mutable), env);
                },
                (Some(name), Some(t), None) => {
                    let v = self.builder.build_alloca(into_basic_type(t).unwrap(), name.as_str());
                    self.push_identi_values(name, VType::new(Some(v.as_any_value_enum()), None, HasType::Type(t), is_mutable), env);
                },
                (Some(name), Some(t), Some(exp)) => {
                    let v = self.builder.build_alloca(into_basic_type(t).unwrap(), name.as_str());
                    self.builder.build_store(v, into_basic_value(exp).unwrap());
                    self.push_identi_values(name, VType::new(Some(v.as_any_value_enum()), None, HasType::Type(t), is_mutable), env);
                },
                (None, Some(_t), _) => {
                    return Err(format!("_ 不能被声明类型"));
                },
                // 左部 _ ，exp 已经解析过
                (None, None, Some(_exp)) => (),
                (n, t, e) => {
                    return Err(format!("{:?} {:?} {:?} 不被变量声明接受", n, t, e));
                }
            }
        }

        Ok(AnyTypeEnum::StructType(self.get_void_type()))
    }

    /// 处理表达式语义，ExecExp
    /// 此时不注册新类型
    /// 后序遍历是中缀表达式
    /// 返回表达式类型和类型名称
    /// （这一块放在语法分析即搞一个语法制导翻译方案会更好，不需要再次遍历一遍计算运算关系和顺序。。。，目前为了方便修改先分离出来）
    fn resolv_exec_exp(&mut self, root: &NodeId, env: &NodeId, func: &FunctionValue) -> Result<AnyValueEnum<'ctx>, String> {
        // 不包含自身，否则会无限递归爆栈
        let exp_childs = self.children_ids(root);
        let nodes: Vec<NodeId> = exp_childs.into_iter().flat_map(|r| {post_traversal_retain_and_skipsubtree(
            self.ast,
            &r,
            |x| {
                !matches!(x,
                    ASTNode::T(Tokens::OrS) | ASTNode::T(Tokens::AndS) | 
                    ASTNode::T(Tokens::Or) | ASTNode::T(Tokens::And) | 
                    ASTNode::T(Tokens::Eq) | ASTNode::T(Tokens::Ne) | 
                    ASTNode::T(Tokens::Gt) | ASTNode::T(Tokens::Lt) | 
                    ASTNode::T(Tokens::Ge) | ASTNode::T(Tokens::Le) | 
                    ASTNode::T(Tokens::Plus) | ASTNode::T(Tokens::Minus) | 
                    ASTNode::T(Tokens::Mul) | ASTNode::T(Tokens::Div) | 
                    ASTNode::T(Tokens::Mod) | ASTNode::T(Tokens::Negate) | 
                    ASTNode::T(Tokens::Str(_)) | ASTNode::T(Tokens::Int(_)) | 
                    ASTNode::T(Tokens::Decimal(_)) | ASTNode::T(Tokens::Bool(_)) | 
                    ASTNode::NT(NT::ExecMatch) | ASTNode::NT(NT::ExecVar) | 
                    ASTNode::NT(NT::ExecExp)
                )
            },
            |x| {
                // 不对其子树进行遍历
                matches!(x,
                    ASTNode::NT(NT::ExecMatch) | 
                    ASTNode::NT(NT::ExecVar) | 
                    ASTNode::NT(NT::ExecExp)
                )
            },
        )}).collect();
        // 定义栈，处理运算顺序（运算符号优先级等）
        let mut ops: Vec<&ASTNode> = vec![];
        let mut nums: Vec<AnyValueEnum> = vec![];
        // _ 不能直接参加运算
        // 由于排除了 ExecMatch / ExecVar / ExecExp 继续遍历下去的可能，这三个符号交由对应的函数处理即可，括号内是 ExecExp，因此可以忽略掉括号
        let mut itr = nodes.iter().enumerate();
        while let Some((index, n)) = itr.next() {
            // 栈操作，获取当前符号
            let judge_n = self.node_data(n);
            let top = match judge_n {
                ASTNode::T(Tokens::OrS) | ASTNode::T(Tokens::AndS) |
                ASTNode::T(Tokens::Or) | ASTNode::T(Tokens::And) |
                ASTNode::T(Tokens::Eq) | ASTNode::T(Tokens::Ne) |
                ASTNode::T(Tokens::Gt) | ASTNode::T(Tokens::Lt) |
                ASTNode::T(Tokens::Ge) | ASTNode::T(Tokens::Le) | 
                ASTNode::T(Tokens::Mul) | ASTNode::T(Tokens::Div) |
                ASTNode::T(Tokens::Mod) => {Some(judge_n)}
                ASTNode::T(Tokens::Plus) => {
                    let d = self.node_data(&nodes[index + 1]);
                    match d {
                        ASTNode::T(Tokens::Str(_)) |
                        ASTNode::T(Tokens::Int(_)) |
                        ASTNode::T(Tokens::Decimal(_)) |
                        ASTNode::T(Tokens::Bool(_)) => {
                            nums.push(self.basic_value(d)?);
                            itr.next();
                            Some(judge_n)
                        }
                        ASTNode::NT(NT::ExecMatch) => {
                            nums.push(self.def_match(&nodes[index + 1], env, func)?);
                            itr.next();
                            Some(judge_n)
                        }
                        ASTNode::NT(NT::ExecVar) => {
                            nums.push(self.resolv_exec_var(&nodes[index + 1], env, func)?.1.unwrap());
                            itr.next();
                            Some(judge_n)
                        }
                        ASTNode::NT(NT::ExecExp) => {
                            nums.push(self.resolv_exec_exp(&nodes[index + 1], env, func)?);
                            itr.next();
                            Some(judge_n)
                        },
                        _ => Some(judge_n)
                    }
                }
                ASTNode::T(Tokens::Minus) => {
                    let d = self.node_data(&nodes[index + 1]);
                    match d {
                        ASTNode::T(Tokens::Str(_)) |
                        ASTNode::T(Tokens::Int(_)) |
                        ASTNode::T(Tokens::Decimal(_)) |
                        ASTNode::T(Tokens::Bool(_)) => {
                            let pr = self.basic_value(d)?;
                            nums.push(self.gen_op(pr, judge_n, None, func)?);
                            itr.next();
                            Some(&ASTNode::T(Tokens::Plus))
                        }
                        ASTNode::NT(NT::ExecMatch) => {
                            let pr = self.def_match(&nodes[index + 1], env, func)?;
                            nums.push(self.gen_op(pr, judge_n, None, func)?);
                            itr.next();
                            Some(&ASTNode::T(Tokens::Plus))
                        }
                        ASTNode::NT(NT::ExecVar) => {
                            let pr = self.resolv_exec_var(&nodes[index + 1], env, func)?.1.unwrap();
                            nums.push(self.gen_op(pr, judge_n, None, func)?);
                            itr.next();
                            Some(&ASTNode::T(Tokens::Plus))
                        }
                        ASTNode::NT(NT::ExecExp) => {
                            let pr = self.resolv_exec_exp(&nodes[index + 1], env, func)?;
                            nums.push(self.gen_op(pr, judge_n, None, func)?);
                            itr.next();
                            Some(&ASTNode::T(Tokens::Plus))
                        },
                        _ => Some(judge_n)
                    }
                }
                ASTNode::T(Tokens::Negate) => {
                    // 直接进行运算
                    let noden = itr.next().unwrap().1;
                    let d = self.node_data(noden);
                    match d {
                        ASTNode::T(Tokens::Str(_)) |
                        ASTNode::T(Tokens::Int(_)) |
                        ASTNode::T(Tokens::Decimal(_)) |
                        ASTNode::T(Tokens::Bool(_)) => {
                            let pr = self.basic_value(d)?;
                            nums.push(self.gen_op(pr, judge_n, None, func)?);
                            None
                        }
                        ASTNode::NT(NT::ExecMatch) => {
                            let pr = self.def_match(noden, env, func)?;
                            nums.push(self.gen_op(pr, judge_n, None, func)?);
                            None
                        }
                        ASTNode::NT(NT::ExecVar) => {
                            let pr = self.resolv_exec_var(noden, env, func)?.1.unwrap();
                            nums.push(self.gen_op(pr, judge_n, None, func)?);
                            None
                        }
                        ASTNode::NT(NT::ExecExp) => {
                            let pr = self.resolv_exec_exp(noden, env, func)?;
                            nums.push(self.gen_op(pr, judge_n, None, func)?);
                            None
                        },
                        _ => return Err(format!("语法分析：单目运算符 ! 解析出错"))
                    }
                }
                ASTNode::T(Tokens::Str(_)) |
                ASTNode::T(Tokens::Int(_)) |
                ASTNode::T(Tokens::Decimal(_)) |
                ASTNode::T(Tokens::Bool(_)) => {
                    nums.push(self.basic_value(judge_n)?);
                    None
                }
                ASTNode::NT(NT::ExecMatch) => {
                    nums.push(self.def_match(n, env, func)?);
                    None
                }
                ASTNode::NT(NT::ExecVar) => {
                    // 只取具体值，此时判断是否返回了 _
                    let r = self.resolv_exec_var(n, env, func)?;
                    nums.push(match r {
                        (Some(_), Some(_), _, Some(_)) |
                        (None, Some(_), _, None) => {
                            r.1.unwrap()
                        },
                        (_, _, _, _) => {
                            return Err(format!("符号不能参与计算：{:?}", r));
                        }
                    });
                    None
                }
                ASTNode::NT(NT::ExecExp) => {
                    nums.push(self.resolv_exec_exp(n, env, func)?);
                    None
                }
                _ => return Err(format!("错误表达式语法解析")),
            };
            // 对顶层符号进行操作，使用 gen_op
            if let Some(x) = top {
                // 不断迭代直至栈为空，或者当前符号优先级高于栈顶
                while !ops.is_empty() && judge_left_gt_right_priority(ops.last().unwrap(), x) && nums.len() > 1 {
                    let op = ops.pop().unwrap();
                    if let (Some(rhs), Some(lhs)) = (nums.pop(), nums.pop()) {
                        let pr = self.gen_op(lhs, op, Some(rhs), func)?;
                        nums.push(pr);
                    } else {
                        return Err(format!("操作数缺失"));
                    }
                }
                ops.push(x);
            }
        }
        // 最后清空栈
        while !ops.is_empty() && nums.len() > 1 {
            let op = ops.pop().unwrap();
            if let (Some(rhs), Some(lhs)) = (nums.pop(), nums.pop()) {
                let pr = self.gen_op(lhs, op, Some(rhs), func)?;
                nums.push(pr);
            } else {
                return Err(format!("操作数缺失"));
            }
        }

        Ok(nums.pop().unwrap())
    }

    /// 处理 Break 语义，ExecBreak
    /// 此时不注册新类型
    fn resolv_exec_break(
        &self,
        root: &NodeId,
        env: &NodeId,
        basicb: Option<BasicBlock>,
    ) -> Result<AnyTypeEnum<'ctx>, String> {
        match basicb {
            Some(x) => self.builder.build_unconditional_branch(x),
            None => {
                return Err(format!("break 未提供跳转块"));
            }
        };

        Ok(AnyTypeEnum::StructType(self.get_void_type()))
    }

    /// 根据 lhs、rhs、op 生成对应的表达式
    /// 实现的源语言中并没有指针（运行阶段确定的字符串除外）
    /// 编译期间确定的字符串可以直接用 ArrayType（i8）确定，分配在静态数据区域
    /// 变量还有结构体（具名的结构体类型）、元组（不具名的结构体类型）、数组（数组类型），这些都不是指针类型，符号表中需要额外的信息
    /// 2023.6.2 目前只准备支持最基本的基本类型操作，即结构体、数组、元组整体都不能直接进行操作，需要进行一下解构，比如数组 var[1] 可能代表的是 IntValue，此时可以计算
    fn gen_op(
        &self,
        lhs: AnyValueEnum<'ctx>,
        op: &ASTNode,
        rhs: Option<AnyValueEnum<'ctx>>,
        func: &FunctionValue,
    ) -> Result<AnyValueEnum<'ctx>, String> {
        match (lhs, op, rhs) {
            (AnyValueEnum::IntValue(l), ASTNode::T(Tokens::Negate), None) => {
                // 得到 i1 type，但还是需要返回 i32 type
                // 2023.6.7，现在类型检测还是有点贫瘠，需要支持多一点，Int 等都全部写死了
                let i1_type = self.context.bool_type();
                let i32_type = self.context.i32_type();
                let ret = self.builder.build_int_truncate(l, i1_type, "int_negate");
                let ret = self.builder.build_int_z_extend(ret, i32_type, "i1_to_i32");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (AnyValueEnum::IntValue(_), ASTNode::T(Tokens::Plus), None)
            | (AnyValueEnum::FloatValue(_), ASTNode::T(Tokens::Plus), None) => Ok(lhs),
            (AnyValueEnum::IntValue(l), ASTNode::T(Tokens::Minus), None) => {
                let ret = self.builder.build_int_neg(l, "intneg");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (AnyValueEnum::FloatValue(l), ASTNode::T(Tokens::Minus), None) => {
                let ret = self.builder.build_float_neg(l, "fneg");
                Ok(AnyValueEnum::FloatValue(ret))
            }
            (o1, o2, None) => {
                Err(format!("lhs: {:?}, op: {:?} 无法进行单目运算", o1, o2))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::OrS),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                // 逻辑 Or
                let i1_type = self.context.bool_type();
                let i32_type = self.context.i32_type();
                let l = self.builder.build_int_truncate(l, i1_type, "int_negate");
                let r = self.builder.build_int_truncate(r, i1_type, "int_negate");
                let true_block = self
                    .context
                    .append_basic_block(*func, "true_block");
                let false_block = self
                    .context
                    .append_basic_block(*func, "false_block");
                let merge_block = self
                    .context
                    .append_basic_block(*func, "merge_block");
                self.builder
                    .build_conditional_branch(l, true_block, false_block);

                self.builder.position_at_end(true_block);
                let true_result = i1_type.const_int(1, false);
                self.builder.build_unconditional_branch(merge_block);

                self.builder.position_at_end(false_block);
                let false_result = r;
                self.builder.build_unconditional_branch(merge_block);

                // phi 基本块内传值
                self.builder.position_at_end(merge_block);
                let result = self.builder.build_phi(i1_type, "or_result");
                result.add_incoming(&[(&true_result, true_block), (&false_result, false_block)]);

                let ret = result.as_any_value_enum();
                let ret = self.builder.build_int_z_extend(ret.into_int_value(), i32_type, "i1_to_i32").as_any_value_enum();
                Ok(ret)
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Or),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                // 算数 Or
                let ret = self.builder.build_or(l, r, "intalgoor");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::AndS),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                // 逻辑 And
                let i1_type = self.context.bool_type();
                let i32_type = self.context.i32_type();
                let l = self.builder.build_int_truncate(l, i1_type, "int_negate");
                let r = self.builder.build_int_truncate(r, i1_type, "int_negate");
                let true_block = self
                    .context
                    .append_basic_block(*func, "true_block");
                let false_block = self
                    .context
                    .append_basic_block(*func, "false_block");
                let merge_block = self
                    .context
                    .append_basic_block(*func, "merge_block");
                self.builder
                    .build_conditional_branch(l, true_block, false_block);

                self.builder.position_at_end(true_block);
                let result_true = r;
                self.builder.build_unconditional_branch(merge_block);

                self.builder.position_at_end(false_block);
                let result_false = i1_type.const_int(0, false);
                self.builder.build_unconditional_branch(merge_block);

                // phi 基本块内传值
                self.builder.position_at_end(merge_block);
                let result = self.builder.build_phi(i1_type, "and_result");
                result.add_incoming(&[(&result_true, true_block), (&result_false, false_block)]);

                let ret = result.as_any_value_enum();
                let ret = self.builder.build_int_z_extend(ret.into_int_value(), i32_type, "i1_to_i32").as_any_value_enum();
                Ok(ret)
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::And),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                // 算数 And
                let ret = self.builder.build_and(l, r, "intalgoand");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Eq),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                let ret = self
                    .builder
                    .build_int_compare(inkwell::IntPredicate::EQ, l, r, "inteq");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Eq),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                // 2023.6.2 这里先选取 OEQ 了
                let ret =
                    self.builder
                        .build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "floateq");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Ne),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                let ret = self
                    .builder
                    .build_int_compare(inkwell::IntPredicate::NE, l, r, "intne");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Ne),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                let ret =
                    self.builder
                        .build_float_compare(inkwell::FloatPredicate::ONE, l, r, "floatne");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Gt),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                // 使用有符号，故使用 Signed GT
                let ret = self
                    .builder
                    .build_int_compare(inkwell::IntPredicate::SGT, l, r, "intgt");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Gt),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                let ret =
                    self.builder
                        .build_float_compare(inkwell::FloatPredicate::OGT, l, r, "floatgt");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Lt),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                let ret = self
                    .builder
                    .build_int_compare(inkwell::IntPredicate::SLT, l, r, "intlt");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Lt),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                let ret =
                    self.builder
                        .build_float_compare(inkwell::FloatPredicate::OLT, l, r, "floatlt");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Ge),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                let ret = self
                    .builder
                    .build_int_compare(inkwell::IntPredicate::SGE, l, r, "intge");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Ge),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                let ret =
                    self.builder
                        .build_float_compare(inkwell::FloatPredicate::OGE, l, r, "floatge");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Le),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                let ret = self
                    .builder
                    .build_int_compare(inkwell::IntPredicate::SLE, l, r, "intle");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Le),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                let ret =
                    self.builder
                        .build_float_compare(inkwell::FloatPredicate::OLE, l, r, "floatle");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Plus),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                let ret = self.builder.build_int_add(l, r, "intadd");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Plus),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                let ret = self.builder.build_float_add(l, r, "floatadd");
                Ok(AnyValueEnum::FloatValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Minus),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                let mr = self.builder.build_int_neg(r, "intneg");
                let ret = self.builder.build_int_add(l, mr, "intminus");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Minus),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                let mr = self.builder.build_float_neg(r, "floatneg");
                let ret = self.builder.build_float_add(l, mr, "floatminus");
                Ok(AnyValueEnum::FloatValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Mul),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                let ret = self.builder.build_int_mul(l, r, "intmul");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Mul),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                let ret = self.builder.build_float_mul(l, r, "floatmul");
                Ok(AnyValueEnum::FloatValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Div),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                let ret = self.builder.build_int_signed_div(l, r, "intdiv");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Div),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                let ret = self.builder.build_float_div(l, r, "floatdiv");
                Ok(AnyValueEnum::FloatValue(ret))
            }
            (
                AnyValueEnum::IntValue(l),
                ASTNode::T(Tokens::Mod),
                Some(AnyValueEnum::IntValue(r)),
            ) => {
                let ret = self.builder.build_int_signed_rem(l, r, "intmod");
                Ok(AnyValueEnum::IntValue(ret))
            }
            (
                AnyValueEnum::FloatValue(l),
                ASTNode::T(Tokens::Mod),
                Some(AnyValueEnum::FloatValue(r)),
            ) => {
                let ret = self.builder.build_float_rem(l, r, "intmod");
                Ok(AnyValueEnum::FloatValue(ret))
            }
            (o1, o2, o3) => {
                Err(format!("lhs: {:?}, rhs: {:?}, op: {:?} 目前无法直接进行运算", o1, o3, o2))
            }
        }
    }

    /// 处理赋值语句语义，ExecIs
    /// 此时不注册新类型
    /// 需要额外注意被赋值的左值是否可变（mut），若不可变却被赋值，则报错
    /// 同时需要注意左值是否是可以被赋值的，即是否已经初始化好，这里直接使用，根据 LLVM IR 判断即可。
    /// 直接构建存储表达式，后期类型检查可以通过 LLVM IR 直接进行
    /// 注意 self 可以被引用，若右边表达式出现 _，该符号不能被赋值（对 _ 的判断在 ExecExp 中确定即可，ExecExp 表达式不接受传入 _ 进行运算）
    /// 若左部符号是 _，且使用的是 = ，只构建右部表达式
    fn resolv_exec_is(
        &mut self,
        root: &NodeId,
        env: &NodeId,
        func: &FunctionValue
    ) -> Result<AnyTypeEnum<'ctx>, String> {
        let ids = self.children_ids(root);

        let (var_point, var_value, is_mut, auto_type) = match self.resolv_exec_var(&ids[0], env, func)? {
            (None, Some(vv), _, None) => {
                (None, Some(vv), false, None)
            },
            (Some(vp), Some(vv), is_mut, Some(auto_type)) => {
                (Some(vp), Some(vv), is_mut, Some(auto_type))
            },
            (Some(vp), Some(vv), _, None) => {
                (Some(vp), Some(vv), false, None)
            },
            (None, None, _, Some(auto_type)) => {
                (None, None, true, Some(auto_type))
            },
            (vp, vv, is_mut, auto_type) => {
                return Err(format!("非左值：{:?} {:?} {:?} {:?}", vp, vv, is_mut, auto_type));
            }
        };
        let op = self.node_data(&self.children_ids(&ids[1])[0]);

        match (op, is_mut, auto_type, var_point, var_value) {
            (ASTNode::T(Tokens::Is), _, Some(HasType::None(inst)), None, None) => {
                // 2023.6.5 目前先搁置实现，这块从文法上可能就对实现有影响
                return Err(format!("暂不支持延迟初始化"));
                let next_b = self.context.append_basic_block(*func, "next_b");
                self.builder.build_unconditional_branch(next_b);
                self.builder.position_before(&inst);

                let exp = self.resolv_exec_exp(&ids[2], env, func)?;
                let init_t = exp.get_type();

                let latec = self.builder.build_alloca(into_basic_type(init_t).unwrap(), "late_init");
                self.builder.position_at_end(next_b);
                self.builder.build_store(
                    latec.as_any_value_enum().into_pointer_value(),
                    into_basic_value(exp).unwrap(),
                );
                // 修改符号表
                // var_vt.has_type = HasType::Type(init_t.to_string());
                // var_vt.value = Some(latec.as_any_value_enum());
            },
            (_, _, None, None, Some(x)) => {
                return Err(format!("无引用的常量值或函数调用等产生的值：{:?}，无法被赋值", x))
            },
            (ASTNode::T(Tokens::Is), _, None, Some(_), Some(_)) => {
                // 遇上 _ ，只构建右表达式（函数使用等）
                self.resolv_exec_exp(&ids[2], env, func)?;
            }
            (o1, _, None, Some(_), Some(_)) => {
                return Err(format!("_ 无法用于 = 以外的赋值表达式，当前运算符号为：{:?}", o1));
            },
            (ASTNode::T(Tokens::PlusIs), true, Some(HasType::Type(_)), Some(var_point), Some(var_value)) => {
                let exp = self.resolv_exec_exp(&ids[2], env, func)?;
                let new_exp = self.gen_op(var_value, &ASTNode::T(Tokens::Plus), Some(exp), func)?;
                self.builder.build_store(
                    var_point.into_pointer_value(),
                    into_basic_value(new_exp).unwrap(),
                );
            },
            (ASTNode::T(Tokens::MinusIs), true, Some(HasType::Type(_)), Some(var_point), Some(var_value)) => {
                let exp = self.resolv_exec_exp(&ids[2], env, func)?;
                let new_exp = self.gen_op(var_value, &ASTNode::T(Tokens::Minus), Some(exp), func)?;
                self.builder.build_store(
                    var_point.into_pointer_value(),
                    into_basic_value(new_exp).unwrap(),
                );
            },
            (ASTNode::T(Tokens::DivIs), true, Some(HasType::Type(_)), Some(var_point), Some(var_value)) => {
                let exp = self.resolv_exec_exp(&ids[2], env, func)?;
                let new_exp = self.gen_op(var_value, &ASTNode::T(Tokens::Div), Some(exp), func)?;
                self.builder.build_store(
                    var_point.into_pointer_value(),
                    into_basic_value(new_exp).unwrap(),
                );
            },
            (ASTNode::T(Tokens::MulIs), true, Some(HasType::Type(_)), Some(var_point), Some(var_value)) => {
                let exp = self.resolv_exec_exp(&ids[2], env, func)?;
                let new_exp = self.gen_op(var_value, &ASTNode::T(Tokens::Mul), Some(exp), func)?;
                self.builder.build_store(
                    var_point.into_pointer_value(),
                    into_basic_value(new_exp).unwrap(),
                );
            },
            (ASTNode::T(Tokens::ModIs), true, Some(HasType::Type(_)), Some(var_point), Some(var_value)) => {
                let exp = self.resolv_exec_exp(&ids[2], env, func)?;
                let new_exp = self.gen_op(var_value, &ASTNode::T(Tokens::Mod), Some(exp), func)?;
                self.builder.build_store(
                    var_point.into_pointer_value(),
                    into_basic_value(new_exp).unwrap(),
                );
            },
            (ASTNode::T(Tokens::Is), true, Some(HasType::Type(_)), Some(var_point), Some(_)) => {
                let exp = self.resolv_exec_exp(&ids[2], env, func)?;              
                self.builder.build_store(
                    var_point.into_pointer_value(),
                    into_basic_value(exp).unwrap(),
                );
            },
            (op, is_mut, vt, vp, vv) => {
                return Err(format!("赋值错误：{:?} {:?} {:?} {:?} {:?}，请注意引用对象和对象是否可变", op, is_mut, vt, vp, vv));
            }
        }

        Ok(AnyTypeEnum::StructType(self.get_void_type()))
    }

    /// 处理 Match 匹配表达式，ExecMatch
    /// 此时不注册新类型
    /// 需要返回值，判断各个块的返回值相同
    /// 需要判断匹配表达式和各个选项类型一致
    /// 然后和各个选项组合成各个选择分支的判断语句
    fn def_match(&mut self, root: &NodeId, env: &NodeId, func: &FunctionValue) -> Result<AnyValueEnum<'ctx>, String> {
        let ids = self.children_ids(root);
        let left = self.resolv_exec_exp(&ids[0], env, func)?;
        let nodes = post_traversal_retain_and_skipsubtree(self.ast.borrow(), &ids[1], |x| {
            !matches!(x,
                ASTNode::NT(NT::ExecExp) |
                ASTNode::NT(NT::FnBody)
            )
        }, |x| {
            matches!(x,
                ASTNode::NT(NT::ExecExp) |
                ASTNode::NT(NT::FnBody)
            )
        });
        // 迭代，每次遇到一个 ExecExp，后面必然是 FnBody，可以根据此弄出各个条件分支跳转
        let mut itr = nodes.into_iter();
        let jump_block = self.context.append_basic_block(*func, "jump_block");
        let mut cases = vec![];
        let mut fns = vec![];
        while let Some(n) = itr.next() {
            match self.node_data(&n) {
                ASTNode::NT(NT::ExecExp) => {
                    // 使用 build_switch，构建出 match 表达式
                    // 1. 左部判断条件 IntValue
                    // 2. 全部未判定成功后的跳转块
                    // 3. 数组，每个元素为右部判断条件和对应跳转块的组合
                    // 首先创建 switch 跳转语句，为此需要先提前解析出每个 exp，创建各个需要的作用域、基本块
                    // 然后针对每个基本块，解析各个 FnBody
                    // 为每一个 FnBody 创建一个新作用域
                    let new_env = self.symbols.create_env(env);
                    let new_block = self.context.append_basic_block(*func, "switch_block");
                    // 先进行解析
                    let exp_v = self.resolv_exec_exp(&n, env, func)?;
                    // 先存储
                    let func_n = itr.next().unwrap();
                    cases.push((exp_v.into_int_value(), new_block));
                    fns.push((func_n, new_env, new_block));
                },
                other => {
                    return Err(format!("match 匹配，符号 {:?} 未被处理", other));
                }
            }
        }
        // 创建中间存储
        let match_ty = into_basic_type(left.get_type()).unwrap();
        let result = self.builder.build_alloca(match_ty, "match_result");
        // 构建 switch 跳转，IR 可以做类型判断
        self.builder.build_switch(left.into_int_value(), jump_block, &cases);
        // 针对每一个 FnBody 进行解析
        for (funcn, new_env, new_b) in fns {
            self.builder.position_at_end(new_b);
            // 不允许使用 break 跳出 match
            self.resolv_fn_body(&funcn, &new_env, None, func, Some(&result), false)?;
            // 每个 switch 跳回 jump_block
            self.builder.build_unconditional_branch(jump_block);
        }
        self.builder.position_at_end(jump_block);
        let result_value = self.builder.build_load(match_ty, result, "match_load_result");

        // 最后返回
        Ok(result_value.as_any_value_enum())
    }

    /// 处理 IF 条件分支语句，ExecIf
    /// 等同于函数
    /// 此时不注册新类型
    /// 放在一次遍历的语法制导翻译方案中更好
    fn def_if(
        &mut self,
        root: &NodeId,
        env: &NodeId,
        basicb: Option<BasicBlock>,
        func: &FunctionValue,
    ) -> Result<AnyTypeEnum<'ctx>, String> {
        let nodes = post_traversal_retain_and_skipsubtree(
            self.ast.borrow(),
            root,
            |x| !matches!(x, ASTNode::NT(NT::ExecExp) | ASTNode::NT(NT::FnBody)),
            |x| matches!(x, ASTNode::NT(NT::ExecExp) | ASTNode::NT(NT::FnBody)),
        );
        // If 的条件判断语句仍然处于给定的当前作用域中（因此符号也是当前作用域的）
        let merge_block = self.context.append_basic_block(*func, "merge_block");
        let mut ret = AnyTypeEnum::StructType(self.get_void_type());
        // 分析的时候判断顺序，Else 分支需要在最后，不在最后直接报错
        // 遇到 ExecExp，直接把后面的 FnBody 连着解析了
        // 遇到 FnBody，表示已经到 Else 块，先判断后续还有没有，还有的话表示顺序错误（Else 块必须放在最后）
        let mut itr = nodes.iter().enumerate();
        while let Some((index, n)) = itr.next() {
            match self.node_data(n) {
                ASTNode::NT(NT::ExecExp) => {
                    let fn_node = itr.next().unwrap().1;
                    let j = self.resolv_exec_exp(n, env, func)?;
                    // 先判断是否可作为判断条件
                    if !j.is_int_value() {
                        return Err(format!("if 分支条件必须为 IntValue，当前条件：{:?}", j));
                    }
                    let new_env = self.symbols.create_env(env);
                    let new_block = self.context.append_basic_block(*func, "elif_block");
                    let jump_block = self.context.append_basic_block(*func, "next_else_control_block");
                    self.builder.build_conditional_branch(j.into_int_value(), new_block, jump_block);
                    self.builder.position_at_end(new_block);
                    let (fn_ret, broken_switch) = self.resolv_fn_body(fn_node, &new_env, basicb, func, None, false)?;
                    // 判断返回类型（return 语句，不能 return 不一样的类型）
                    if fn_ret.eq(&AnyTypeEnum::StructType(self.get_void_type())) {} 
                    else if ret.eq(&AnyTypeEnum::StructType(self.get_void_type())) || 
                        same_type(&ret, &fn_ret) {
                        ret = fn_ret;
                    } else {
                        return Err(format!("If 主和 Else If 分支返回类型冲突，应返回类型：{:?}，实际返回类型：{:?}", ret, fn_ret));
                    }
                    // 每一个分支结束后，若没有构建过 ret，都需要跳转至 merge_block（已经构建了 build_return 不用管，会被忽略）
                    if !broken_switch {
                        // self.builder.position_at_end(new_block);
                        self.builder.build_unconditional_branch(merge_block);
                    }
                    // 到下一分支进行工作
                    self.builder.position_at_end(jump_block);
                    // 当已经没有其他语句控制块时，jump_block 跳至 merge_block
                    if nodes.get(index + 2).is_none() {
                        self.builder.build_unconditional_branch(merge_block);
                    }
                },
                ASTNode::NT(NT::FnBody) => {
                    // 构建 else 语句
                    if let Some(x) = itr.next() {
                        let e = self.node_data(x.1);
                        return Err(format!("If 的 Else 块后不应添加任何条件判断分支：{:?}", e));
                    } else {
                        // Else 块，可以直接构建语句，最后跳到 merge_block 即可
                        let new_env = self.symbols.create_env(env);
                        // 不允许 break 语句
                        let (fn_ret, broken_switch) = self.resolv_fn_body(n, &new_env, basicb, func, None, false)?;
                        if fn_ret.eq(&AnyTypeEnum::StructType(self.get_void_type())) {} 
                        else if ret.eq(&AnyTypeEnum::StructType(self.get_void_type())) || 
                            same_type(&ret, &fn_ret) {
                            ret = fn_ret;
                        } else {
                            return Err(format!("If Else 分支返回类型冲突，应返回类型：{:?}，实际返回类型：{:?}", ret, fn_ret));
                        }
                        if !broken_switch {
                            self.builder.build_unconditional_branch(merge_block);
                        }
                    }
                },
                other => {
                    return Err(format!("If 语义分析无法分析块：{:?}", other));
                }
            }
        }
        // 最后跳转至 merge_block 进行后续分析
        self.builder.position_at_end(merge_block);

        Ok(ret)
    }

    /// 处理 While 循环语句，ExecWhile
    /// 等同于函数
    /// 此时不注册新类型
    fn def_while(
        &mut self,
        root: &NodeId,
        env: &NodeId,
        // basicb: Option<BasicBlock>,
        func: &FunctionValue,
    ) -> Result<AnyTypeEnum<'ctx>, String> {
        let ids = self.children_ids(root);
        let mut ret = AnyTypeEnum::StructType(self.get_void_type());
        let broken_switch;

        let exp_block = self
        .context
        .append_basic_block(*func, "exp_block");
        let while_block = self.context.append_basic_block(*func, "while_block");
        let jump_block = self.context.append_basic_block(*func, "jump_block");
        let while_env = self.symbols.create_env(env);
        // 当前块跳转至判断
        self.builder.build_unconditional_branch(exp_block);
        // 构建判断块
        self.builder.position_at_end(exp_block);
        // 每次都需要重新计算一次
        let exp = self.resolv_exec_exp(&ids[0], env, func)?;
        // 先判断是否可作为判断条件
        if !exp.is_int_value() {
            return Err(format!("while 判断条件必须为 IntValue，当前条件：{:?}", exp));
        }
        self.builder.build_conditional_branch(exp.into_int_value(), while_block, jump_block);
        self.builder.position_at_end(while_block);
        // 返回值
        (ret, broken_switch) = self.resolv_fn_body(&ids[1], &while_env, Some(jump_block), func, None, false)?;
        if !broken_switch {
            // self.builder.position_at_end(while_block);
            self.builder.build_unconditional_branch(exp_block);
        }
        self.builder.position_at_end(jump_block);

        Ok(ret)
    }

    /// 处理 Loop 循环语句，ExecLoop
    /// 此时不注册新类型
    fn def_loop(
        &mut self,
        root: &NodeId,
        env: &NodeId,
        // basicb: Option<BasicBlock>,
        func: &FunctionValue,
    ) -> Result<AnyTypeEnum<'ctx>, String> {
        let ids = self.children_ids(root);
        let mut ret = AnyTypeEnum::StructType(self.get_void_type());
        let broken_switch;

        let loop_block = self.context.append_basic_block(*func, "loop_block");
        let exp_block = self.context.append_basic_block(*func, "exp_block");
        let jump_block = self.context.append_basic_block(*func, "jump_block");
        let loop_env = self.symbols.create_env(env);
        self.builder.build_unconditional_branch(loop_block);
        self.builder.position_at_end(loop_block);
        // 返回值
        (ret, broken_switch) = self.resolv_fn_body(&ids[0], &loop_env, Some(jump_block), func, None, false)?;
        if !broken_switch {
            // self.builder.position_at_end(loop_block);
            self.builder.build_unconditional_branch(exp_block);
        }
        self.builder.position_at_end(exp_block);
        // 每次计算一遍
        let exp = self.resolv_exec_exp(&ids[1], env, func)?;
        // 先判断是否可作为判断条件
        if !exp.is_int_value() {
            return Err(format!("loop 判断条件必须为 IntValue，当前条件：{:?}", exp));
        }
        self.builder.position_at_end(exp_block);
        self.builder.build_conditional_branch(exp.into_int_value(), loop_block, jump_block);
        self.builder.position_at_end(jump_block);

        Ok(ret)
    }

    /// 处理成员引用表达式语义，ExecVar
    /// 从左到右：
    /// 0. 查找符号表等查看是否存在（存在该符号，结构体有此引用），获取其值（类型）
    /// 1. 每次判断是否允许对上一个类型进行该操作
    /// 2. 生成新类型
    /// 3. 最后得到(引用的指针值，引用本身存储的具体值)
    /// 此时不注册新类型
    /// 返回对象的 PointerValue（指针），对象引用代表的值，对象对应的指示
    /// 特判：
    /// 1. _ （特殊标识符，需要额外做处理）
    /// 2. read，write（核心库的一部分）-> 其实可以在解析开始前将这些库加入符号表
    /// 后续如果做多模块，其实相当于词法阶段做一些解析，多个模块之间根据关系组成图，按照拓扑序不断构造，前面的相关函数、类型等根据规则加入后面的符号表等进行使用
    /// 3. _() 空元组，表示为常量
    /// self 可以不用，直接按照普通量进行符号表查找就行，只是不能对 self 做定义等
    /// vp, vv, is_mut(bool), Option<HashType> 
    /// 2023.6.6，延迟初始化未加入，暂不传递 vt(VType)
    /// 除了错误情况，对各种可能进行归类：（0 1 3）
    /// var_point var_value auto_type
    /// 1. None, Some, None => 只是值，没有引用（常量、函数调用后的量，未被存储在符号表中），不用管是否可变
    /// 2. None, Some, Some => 不可能
    /// 3. None, None, None => 不可能（啥都没获取到）
    /// 4. None, None, Some => 正常成员引用，但要做类型推导（自动推导，最开始只能保留符号名，还不能存储值和类型名）
    /// 2023.6.6，4 选项目前不实现，即不添加延迟初始化
    /// 5. Some, Some, Some => 正常成员引用，肯定不需要做类型推导
    /// 6. Some, Some, None => 表示 _ （统一返回 int(0)），不用管是否可变
    /// 7. Some, None, Some => 不可能
    /// 8. Some, None, None => 不可能
    fn resolv_exec_var(
        &mut self,
        root: &NodeId,
        env: &NodeId,
        func: &FunctionValue
    ) -> Result<(Option<AnyValueEnum<'ctx>>, Option<AnyValueEnum<'ctx>>, bool, Option<HasType<'ctx>>), String> {
        let nodes = post_traversal_retain_and_skipsubtree(
            self.ast,
            root,
            |x| {
                !matches!(x,
                    ASTNode::T(Tokens::Identity(_)) | ASTNode::T(Tokens::LeftMB) | 
                    ASTNode::T(Tokens::Int(_)) | ASTNode::T(Tokens::RightMB) | 
                    ASTNode::T(Tokens::ShouldReturn) | ASTNode::T(Tokens::LeftC) | 
                    ASTNode::T(Tokens::RightC) | ASTNode::NT(NT::ExecExp)
                )
            },
            |x| matches!(x, ASTNode::NT(NT::ExecExp)),
        );
        
        let mut has_pointer = true;
        let mut vv;
        // 先得到第一个变量的值和对应的类型，这里对 _ 、 _() 、右值进行特判
        let judge_n = self.node_data(&nodes[0]);
        let mut skip = 1;
        let (mut vp, mut vt, is_mut, auto_type) = match judge_n {
            // 判定
            ASTNode::T(Tokens::Identity(x)) => {
                if x.eq("_") {
                    if nodes.len() == 1 {
                        // _ => i32_type().const_zero，不用管是否可变
                        let j = self.basic_value(judge_n)?;
                        return Ok((Some(j.as_any_value_enum()), Some(j.as_any_value_enum()), false, None));
                    } else if nodes.len() >= 3 &&
                        self.node_data(&nodes[1]) == &ASTNode::T(Tokens::LeftC) {
                        if self.node_data(&nodes[2]) == &ASTNode::T(Tokens::RightC) {
                            return Ok((None, Some(AnyValueEnum::StructValue(self.get_void_value())), false, None));
                        } else {
                            // 看是否是创建了元组，如果是创建了元组，相当于只有值，但要继续下去
                            // _(32, 55...)
                            todo!("目前暂不支持创建元组。。。")
                            // skip = ..;
                        }
                    } else {
                        return Err(format!("以 _ 开头的成员引用仅支持：_ 和 _()，当前成员引用为：{:?}", nodes));
                    }
                } else {
                    let constv = self.basic_value(judge_n);
                    // 向上查找
                    let varv = self.identi_lookup_vtype(x, env);

                    if let Ok(v) = constv {
                        // 识别为常量，不用管是否可变
                        return Ok((None, Some(v), false, None));
                    }
                    if let Some(vt) = varv {
                        match (&vt.value, &vt.has_type) {
                            (Some(ptr), HasType::Type(t)) => {
                                if self.judge_left_value(ptr) {
                                    // 创建临时变量
                                    has_pointer = false;
                                    let basict = into_basic_type(*t).unwrap();
                                    let tmp = self.builder.build_alloca(basict, "create_tmp");
                                    self.builder.build_store(tmp, into_basic_value(*ptr).unwrap());
                                    vv = ptr.as_any_value_enum();
                                    (tmp.as_any_value_enum(), *t, false, Some(vt.has_type.clone()))
                                } else {
                                    if ptr.is_pointer_value() {
                                        // PointerValue
                                        vv = self.builder.build_load(into_basic_type(*t).unwrap(), ptr.into_pointer_value(), "init_vv").as_any_value_enum();
                                    } else {
                                        // FunctionValue
                                        vv = ptr.as_any_value_enum();
                                    }
                                    (*ptr, *t, vt.is_mut, Some(vt.has_type.clone()))
                                }
                            },
                            (_, _) => {
                                return Err(format!("成员：{:?}值未被初始化，无法被引用", varv));
                            }
                        }   
                    } else {
                        return Err(format!("成员未找到：{:?}", x));
                    }
                }
            },
            other => return Err(format!("引用第一个单词错误：{:?}", other))
        };

        let mut itr = nodes.iter().enumerate().skip(skip);
        while let Some((index, n)) = itr.next() {
            match self.node_data(n) {
                ASTNode::T(Tokens::LeftMB) => {
                    // 获取 int，语法已保证
                    let i = self.context.i32_type().const_int(*match self.node_data(itr.next().unwrap().1) {
                        ASTNode::T(Tokens::Int(x)) => {
                            x
                        },
                        other => {
                            return Err(format!("语法分析：数组索引识别出错，识别到索引：{:?}", other));
                        }
                    } as u64, false);
                    // 跳过 LeftMB
                    itr.next();
                    let vvv = self.builder.build_load(into_basic_type(vt).unwrap(), vp.into_pointer_value(), "load_ptr");
                    if vvv.is_array_value() {
                        let vvv_t = vvv.into_array_value().get_type().get_element_type();
                        vp = unsafe { self.builder.build_gep(vvv_t, vp.into_pointer_value(), &[i], "load_arr_data_ptr") }.as_any_value_enum();
                        vt = vvv_t.as_any_type_enum();
                        vv = self.builder.build_load(vvv_t, vp.into_pointer_value(), "load_arr_data").as_any_value_enum();
                    } else {
                        return Err(format!("不能对数组以外的符号进行数组索引，当前被引用对象：{:?}", vvv));
                    }
                },
                ASTNode::T(Tokens::LeftC) => {
                    // 判断 vp 是不是函数
                    match vp {
                        AnyValueEnum::FunctionValue(x) => {
                            has_pointer = false;
                            // 进行函数调用，获取形式参数，语法已保证，不需要额外做判断
                            let mut params = vec![];
                            let mut n = itr.next().unwrap();
                            while let ASTNode::NT(NT::ExecExp) = self.node_data(n.1) {
                                // 得到各个形参的引用，首先需要抓取各个形参的量
                                let v = self.resolv_exec_exp(n.1, env, func)?;
                                params.push(into_basic_metadata_value(v).unwrap());
                                n = itr.next().unwrap();
                            }
                            let call_ret = self.builder.build_call(x, &params, "call_func");
                            let call_ret_v = call_ret.try_as_basic_value().left().unwrap().as_basic_value_enum();
                            let bvt = x.get_type().get_return_type().unwrap();
                            let call_tmp = self.builder.build_alloca(bvt, "build_call_ret_tmp");
                            self.builder.build_store(call_tmp, call_ret_v);
                            vt = bvt.as_any_type_enum();
                            vp = call_tmp.as_any_value_enum();
                            vv = call_ret_v.as_any_value_enum();
                        },
                        other => {
                            return Err(format!("不知名函数：{:?}", other))
                        }
                    }
                },
                ASTNode::T(Tokens::ShouldReturn) => {
                    // 判断是 ->0 还是 ->a 还是 ->a(...)
                    let n = itr.next().unwrap();
                    match self.node_data(n.1) {
                        ASTNode::T(Tokens::Int(x)) => {
                            let i = *x as u32;
                            let vvv = self.builder.build_load(into_basic_type(vt).unwrap(), vp.into_pointer_value(), "load_ptr");
                            if vvv.is_struct_value() {
                                let vvv_t = vvv.into_struct_value().get_type().get_field_types();
                                vp = unsafe { self.builder.build_gep(vvv_t[i as usize], vp.into_pointer_value(), &[self.context.i32_type().const_int(i as u64, false)], "load_tuple_data_ptr") }.as_any_value_enum();
                                vt = vvv_t[i as usize].as_any_type_enum();
                                vv = self.builder.build_load(vvv_t[i as usize], vp.into_pointer_value(), "load_tuple_data").as_any_value_enum();
                            } else {
                                return Err(format!("不能对数组以外的符号进行数组索引，当前被引用对象：{:?}", vvv));
                            }
                        },
                        ASTNode::T(Tokens::Identity(x)) => {
                            // 获取结构体相关信息
                            let vvv = self.builder.build_load(into_basic_type(vt).unwrap(), vp.into_pointer_value(), "load_ptr");
                            // 结构体字段内情量
                            let str_params;
                            // 结构体作用域（函数）
                            let str_env;
                            if vvv.is_struct_value() {
                                let st = vvv.into_struct_value().get_type();
                                (str_params, str_env) = match self.identi_lookup_tytype(&st.to_string(), env) {
                                    Some(stty) => {
                                        (stty.params.clone(), stty.has_env.as_ref().unwrap())
                                    },
                                    None => {
                                        return Err(format!("结构体类型信息未被记录，无法进行结构体引用，名称为：{:?}", st));
                                    }
                                }
                            } else {
                                return Err(format!("无法对结构体以外的对象进行字段或结构体函数引用，当前被引用对象：{:?}", vvv));
                            }
                            match &nodes.get(n.0 + 1) {
                                None => {
                                    // 先查看是否在 params 内
                                    if let Some(i) = str_params.iter().position(|&f| f == x) {
                                        // 获取结构体字段;
                                        let vvv_t = vvv.into_struct_value().get_type().get_field_types();
                                        vp = unsafe { self.builder.build_gep(vvv_t[i], vp.into_pointer_value(), &[self.context.i32_type().const_int(i as u64, false)], "load_struct_data_ptr") }.as_any_value_enum();
                                        vt = vvv_t[i].as_any_type_enum();
                                        vv = self.builder.build_load(vvv_t[i], vp.into_pointer_value(), "load_struct_data").as_any_value_enum();
                                        continue;
                                    } else {
                                        return Err(format!("结构体不存在该字段：{:?}", x));
                                    }
                                },
                                Some(_) => ()
                            };
                            match self.node_data(&nodes[n.0 + 1]) {
                                ASTNode::T(Tokens::LeftC) => {
                                    // 解析函数，首先查看是否存在该函数（使用名称）
                                    let str_fun = match self.symbols.looknow_values(x, str_env) {
                                        Some(x) => {
                                            match x.value {
                                                Some(AnyValueEnum::FunctionValue(y)) => y,
                                                _ => {
                                                    return Err(format!("结构体不存在该函数：{:?}", x));
                                                }
                                            }
                                        },
                                        None => return Err(format!("结构体不存在该函数：{:?}", x))
                                    };
                                    let str_fun_p_count = str_fun.count_params();
                                    let mut params = vec![];
                                    itr.next();
                                    let mut n = itr.next().unwrap();
                                    while let ASTNode::NT(NT::ExecExp) = self.node_data(n.1) {
                                        // 得到各个形参的引用，首先需要抓取各个形参的量
                                        let v = self.resolv_exec_exp(n.1, env, func)?;
                                        params.push(into_basic_metadata_value(v).unwrap());
                                        n = itr.next().unwrap();
                                    }
                                    // 由于默认添加（写的时候不用指定，可能导致调用时缺少形参），结构体形参放最后
                                    if str_fun_p_count as usize == params.len() + 1 {
                                        params.push(vvv.into())
                                    }
                                    let call_ret = self.builder.build_call(str_fun, &params, "call_func");
                                    let call_ret_v = call_ret.try_as_basic_value().left().unwrap().as_basic_value_enum();
                                    let bvt = str_fun.get_type().get_return_type().unwrap();
                                    let call_tmp = self.builder.build_alloca(bvt, "build_call_ret_tmp");
                                    self.builder.build_store(call_tmp, call_ret_v);
                                    vt = bvt.as_any_type_enum();
                                    vp = call_tmp.as_any_value_enum();
                                    vv = call_ret_v.as_any_value_enum();
                                },
                                other => {
                                    return Err(format!("结构体除函数调用和字段引用外的其他非法引用: {:?}", other));
                                }
                            }
                        },
                        other => {
                            return Err(format!("-> 引用，后应接数组、字段名称或函数调用，此处引用为：{:?}", other))
                        }
                    }
                },
                other => {
                    return Err(format!("引用格式错误，引用符号：{:?}", other))
                }
            }
        }

        if has_pointer {
            Ok((Some(vp), Some(vv), is_mut, auto_type))
        } else {
            Ok((None, Some(vv), is_mut, None))
        }
    }

    fn get_void_type(&self) -> StructType<'ctx> {
        self.context.struct_type(&[], false)
    }

    fn get_void_value(&self) -> StructValue<'ctx> {
        self.context.struct_type(&[], false).const_named_struct(&[])
    }

    /// 根据 Tokens，判断是否是编译器可确定的常量，返回对应的常量值
    fn basic_value(&self, tok: &ASTNode) -> Result<AnyValueEnum<'ctx>, String> {
        match tok {
            ASTNode::T(Tokens::Int(x)) => {
                // 源语言中没有直接是负数的单元，运算结果可以是负数
                let _x: u64 = match (*x).try_into() {
                    Ok(y) => y,
                    Err(e) => {
                        return Err(format!(
                            "value {} out of compiler's limited range, due to: {}",
                            x, e
                        ));
                    }
                };
                // 2023.6.2 为方便，所有的都先默认为有符号
                Ok(AnyValueEnum::IntValue(
                    self.context.i32_type().const_int(_x, true),
                ))
            },
            ASTNode::T(Tokens::Decimal(x)) => {
                let _x: f64 = match (*x).try_into() {
                    Ok(y) => y,
                    Err(e) => {
                        return Err(format!(
                            "value {} out of compiler's limited range, due to: {}",
                            x, e
                        ));
                    }
                };
                Ok(AnyValueEnum::FloatValue(
                    self.context.f32_type().const_float(_x),
                ))
            },
            ASTNode::T(Tokens::Str(x)) => {
                let c = self.builder.build_global_string_ptr(x, "str");
                Ok(AnyValueEnum::PointerValue(c.as_pointer_value()))
            },
            ASTNode::T(Tokens::Bool(x)) => {
                let v = match x {
                    true => 1,
                    false => 0,
                };
                let t = self.context.bool_type().const_int(v, true);
                Ok(AnyValueEnum::IntValue(t))
            }
            ASTNode::T(Tokens::Identity(x)) => {
                if x.eq("_") {
                    Ok(AnyValueEnum::IntValue(self.context.i32_type().const_zero()))
                } else if x.eq("_()") {
                    Ok(AnyValueEnum::StructValue(self.get_void_value()))
                } else {
                    Err(format!("单元：{:?} 不为常量", x))
                }
            },
            _ => Err(format!("给定单元 {:?} 不为常量", tok))
        }
    }

    /// 根据具体字符，判断是否是基本类型
    /// str: 字符串
    /// i32: 整常数
    /// f32: 小数
    /// bool: 布尔常数
    /// (): 空元组基本类型，空的不具名结构体类型
    /// 返回类型
    fn basic_type(&self, x: &str) -> Result<AnyTypeEnum<'ctx>, String> {
        match x {
            "str" => {
                let r = self.context.i8_type().ptr_type(AddressSpace::default());
                return Ok(AnyTypeEnum::PointerType(r));
            }
            "i32" => {
                let r = self.context.i32_type();
                return Ok(AnyTypeEnum::IntType(r));
            }
            "f32" => {
                let r = self.context.f32_type();
                return Ok(AnyTypeEnum::FloatType(r));
            }
            "bool" => {
                let r = self.context.bool_type();
                return Ok(AnyTypeEnum::IntType(r));
            },
            "()" => {
                let r = self.get_void_type();
                return Ok(AnyTypeEnum::StructType(r));
            }
            _ => (),
        };
        Err(format!("无此基本类型: {}", x))
    }

    #[inline]
    /// 判断左值
    /// 当前先认为函数值也是左值
    fn judge_left_value(&self, v: &AnyValueEnum) -> bool {
        !matches!(v, AnyValueEnum::PointerValue(_) | AnyValueEnum::FunctionValue(_))
    }

    /// 处理类型声明语句语义，ExecType
    /// 此时不注册新类型
    /// 返回类型
    fn resolv_exec_type(&self, root: &NodeId, env: &NodeId) -> Result<AnyTypeEnum<'ctx>, String> {
        let ids = self.children_ids(root);
        match self.node_data(&ids[0]) {
            ASTNode::T(Tokens::Identity(x)) => {
                // 判断是否为基本类型
                if let Ok(r) = self.basic_type(x) {
                    return Ok(r);
                }
                // 判断在当前作用域以及其祖先作用域内是否存在该类型
                if let Some(vt) = self.identi_looknow_types(x, env) {
                    return Ok(vt);
                }
                Err(format!("无此类型：{:?}", x))
            },
            ASTNode::T(Tokens::LeftMB) => {
                let at = self.resolv_exec_type(&ids[1], env)?;
                if let ASTNode::T(Tokens::Int(alen)) = self.node_data(&ids[3]) {
                    let r = at.into_array_type().array_type(*alen as u32);
                    return Ok(AnyTypeEnum::ArrayType(r));
                } else {
                    unreachable!("数组元素个数标识缺失")
                }
            },
            ASTNode::T(Tokens::LeftC) => {
                if ids.len() == 2 {
                    let r = self.get_void_type();
                    return Ok(AnyTypeEnum::StructType(r));
                } else {
                    // 识别元组类型
                    let nodes = post_traversal_retain_and_skipsubtree(
                        self.ast.borrow(),
                        &ids[1],
                        |_x| false,
                        |x| x.eq(&ASTNode::NT(NT::ExecType)),
                    );
                    let mut tuple_inner: Vec<BasicTypeEnum> = vec![];
                    for n in nodes {
                        match self.node_data(&n) {
                            ASTNode::NT(NT::ExecType) => {
                                let c = self.resolv_exec_type(&n, env)?;
                                tuple_inner.push(into_basic_type(c).unwrap());
                            }
                            _ => continue,
                        }
                    }
                    let r = self.context.struct_type(&tuple_inner, false);
                    return Ok(AnyTypeEnum::StructType(r));
                }
            },
            _ => {
                unreachable!("声明类型的语法检查错误")
            }
        }
    }

    /// 处理函数体
    /// 遍历 ExecSentence 调用对应函数进行处理
    /// 需要额外注意返回语句，判断函数体类型，返回函数体返回类型
    /// basicb : 表示在进行 break 等跳转时，需要跳至的块，由 loop、while 等产生
    /// 返回 FnBody 的返回类型，以及是否构建了 break / return 等破坏跳转关系的语句
    fn resolv_fn_body(
        &mut self,
        root: &NodeId,
        env: &NodeId,
        basicb: Option<BasicBlock>,
        func: &FunctionValue,
        // result 表示构建返回语句时，不是 build_return，而是将返回值进行存储
        result: Option<&PointerValue>,
        // FnBody 在没有返回语句构建时，需不需要构建默认返回语句（空元组，表示语句的返回值）
        need_default_void_return: bool
    ) -> Result<(AnyTypeEnum<'ctx>, bool), String> {
        let mut r = AnyTypeEnum::StructType(self.get_void_type());
        // 只需保留 ExecSentence
        let nodes = post_traversal_retain_and_skipsubtree(
            self.ast,
            root,
            |x| !matches!(x, ASTNode::NT(NT::ExecSentence)),
            |x| matches!(x, ASTNode::NT(NT::ExecSentence)),
        );
        let mut is_over = false;
        let mut l;
        let itr = nodes.iter();
        for n in itr {
            let eids = self.children_ids(n);

            (l, is_over) = match self.node_data(&eids[0]) {
                ASTNode::NT(NT::ExecStmt) => {
                    (self.def_var(&eids[0], env, func)?, false)
                },
                ASTNode::NT(NT::ExecIs) => {
                    (self.resolv_exec_is(&eids[0], env, func)?, false)
                },
                ASTNode::NT(NT::ExecIf) => {
                    (self.def_if(&eids[0], env, basicb, func)?, false)
                },
                ASTNode::NT(NT::ExecWhile) => {
                    (self.def_while(&eids[0], env, func)?, false)
                },
                ASTNode::NT(NT::ExecLoop) => {
                    (self.def_loop(&eids[0], env, func)?, false)
                },
                ASTNode::NT(NT::ExecRet) => {
                    match result {
                        None => {
                            let ids = self.children_ids(&eids[0]);
                            let value = self.resolv_exec_exp(&ids[0], env, func)?;
                            let r = into_basic_value(value).unwrap();
                    
                            // 创建返回语句
                            self.builder.build_return(Some(&r as &dyn BasicValue));

                            // 返回类型
                            (value.get_type(), true)
                        },
                        Some(ptrr) => {
                            let ids = self.children_ids(&eids[0]);
                            let value = self.resolv_exec_exp(&ids[0], env, func)?;
                            let r = into_basic_value(value).unwrap();
                            
                            // 构建存储语句保证
                            self.builder.build_store(*ptrr, r);

                            // 返回类型
                            (value.get_type(), true)
                        }
                    }
                },
                ASTNode::NT(NT::ExecBreak) => {
                    // 传递需要跳转的块，只允许 loop、while 中，match 不行，由 def_loop 和 def_while 保证
                    // 分析了 break 后，其他的也不需要分析了，表示 FnBody 分析结束
                    // 之后可以将后续语句标记为 unreachable
                    (self.resolv_exec_break(&eids[0], env, basicb)?, true)   
                }
                _ => unreachable!("不可能语句"),
            };

            // 比较各个其他返回的类型是否相同（return、if/loop/while 中 return 等）
            // 空元组为固定返回类型，不用管，当其本身是空元组时，返回其他类型，第一次转换，第二次判断
            if l.eq(&AnyTypeEnum::StructType(self.get_void_type())) {} 
            else if r.eq(&AnyTypeEnum::StructType(self.get_void_type())) || 
                same_type(&r, &l) {
                r = l;
            } else {
                return Err(format!("返回类型冲突。应返回：{:?}，实际返回：{:?}", r, l));
            }

            if is_over {
                break;
            }
        }
        if need_default_void_return && !is_over {
            // 默认添加 ret ()
            self.builder.build_return(Some(&self.get_void_value().as_basic_value_enum() as &dyn BasicValue));
        }
        Ok((r, is_over))
    }

    #[inline]
    fn get_root_env(&self) -> NodeId {
        self.symbols.root_env()
    }

    #[inline]
    fn identi_lookup_types(&self, id: &str, env: &NodeId) -> Option<AnyTypeEnum> {
        self.symbols.lookup_types(id, env).map(|x| x.type_value)
    }
    
    #[inline]
    fn identi_lookup_tytype(&self, id: &str, env: &NodeId) -> Option<&TyType> {
        self.symbols.lookup_types(id, env)
    }

    #[inline]
    fn identi_lookup_values(&self, id: &str, env: &NodeId) -> Option<AnyValueEnum> {
        match self.symbols.lookup_values(id, env) {
            Some(x) => x.value,
            None => None
        }
    }

    #[inline]
    fn identi_lookup_vtype(&self, id: &str, env: &NodeId) -> Option<&VType<'ctx>> {
        self.symbols.lookup_values(id, env)
    }

    #[inline]
    fn identi_looknow_types(&self, id: &str, env: &NodeId) -> Option<AnyTypeEnum<'ctx>> {
        self.symbols.looknow_types(id, env).map(|x| x.type_value)
    }

    #[inline]
    fn identi_looknow_tytype(&self, id: &str, env: &NodeId) -> Option<&TyType> {
        self.symbols.looknow_types(id, env)
    }

    #[inline]
    fn identi_looknow_values(&self, id: &str, env: &NodeId) -> Option<AnyValueEnum> {
        match self.symbols.looknow_values(id, env) {
            Some(x) => x.value,
            None => None
        }
    }

    #[inline]
    fn identi_looknow_vtype(&self, id: &str, env: &NodeId) -> Option<&VType<'ctx>> {
        self.symbols.looknow_values(id, env)
    }

    #[inline]
    fn push_identi_type(
        &mut self,
        id: &str,
        value: TyType<'ctx>,
        env: &NodeId,
    ) {
        self.symbols.push_symbol_types(id, value, env);
    }

    #[inline]
    fn push_identi_values(
        &mut self,
        id: &str,
        value: VType<'ctx>,
        env: &NodeId,
    ) {
        self.symbols.push_symbol_values(id, value, env);
    }

    #[inline]
    fn node_data(&self, root: &NodeId) -> &'a ASTNode {
        self.ast.get(root).unwrap().data()
    }

    #[inline]
    fn children_ids(&self, root: &NodeId) -> Vec<NodeId> {
        self.ast.children_ids(root).unwrap().cloned().collect()
    }
}

#[cfg(test)]
mod llvmir_gen_tests {
    use std::fs::File;

    use id_tree::{Tree, Node};
    use inkwell::context::Context;

    use crate::{semantic::llvmir_gen::same_type, syntax::{ASTNode, ll_parser::RecursiveDescentParser, NT}, lex::{Tokens, analysis::Analysis, preprocessor::preprocessor}, table::symbol::SymbolManager};

    use super::{post_traversal_retain_and_skipsubtree, IrGen};

    macro_rules! llvmir_gen_test_macro {
        ($file:expr, $test:expr, $retT:ty, $ret:expr, $funcN:expr) => {
            let file_path = format!("examples/sources/{}", $file);
            let mut path = std::env::current_dir().unwrap();
            path.push(file_path);
            let file = File::open(path).unwrap();
            let preprocess = preprocessor(&file);
            let analysis = Analysis::new_with_capacity($file, &preprocess, preprocess.len());
            let mut parser = RecursiveDescentParser::new(analysis).unwrap();
            parser.parse();
            parser.print_test();
            let context = Context::create();
            let mut symbols = SymbolManager::new();
            let mut llvmir_gen_engine = IrGen::new(parser.get_ast(), &context, &mut symbols, $funcN);
            let result = llvmir_gen_engine.gen();
            if let Err(e) = &result {
                println!("{}", e);
                assert!(!$test);
                return;
            } else {
                println!("{}", llvmir_gen_engine.dump());
                assert!($test)
            }
            let verified = llvmir_gen_engine.verify();
            if let Err(e) = &verified {
                println!("{}", e);
                assert!(!$test);
                return;
            } else {
                println!("verified ok!");
                assert!($test)
            }
            let run = llvmir_gen_engine.jit_execute::<$retT>(inkwell::OptimizationLevel::None, "main");
            match (&run, &$ret) {
                (Err(e), _) => println!("{}", e),
                (Ok(l), Some(r)) => {
                    if l.eq(r) {
                        println!("func ret same as expected!");
                        assert!($test)
                    } else {
                        println!("func ret: {}, expected: {}, not same!", l, r);
                        assert!(!$test)
                    }
                },
                _ => ()
            }
        };
    }

    #[test]
    fn test_negate() {
        let context = Context::create();
        let builder = context.create_builder();
        let i32_type = context.i32_type();
        let i1_type = context.bool_type();
    
        // negate 操作最后应该返回 i1.type
        let value = i32_type.const_int(42, false);
        let bool_value = builder.build_int_truncate(value, i1_type, "bool_value");
        let value1 = i32_type.const_int(0, false);
        let bool_value1 = builder.build_int_truncate(value1, i1_type, "bool_value");
    
        println!("Original value: {:?}", value.get_zero_extended_constant());
        println!("Boolean value: {:?}", bool_value.get_zero_extended_constant());
        println!("Original value1: {:?}", value1.get_zero_extended_constant());
        println!("Boolean value1: {:?}", bool_value1.get_zero_extended_constant());
    }

    #[test]
    fn test_same_type() {
        let context = Context::create();
        let t1 = context.i32_type();
        let t2 = context.i64_type();
        let t3 = t1.array_type(3);
        let t4 = t1.array_type(6);
        let t5 = context.struct_type(&[t1.into()], false);
        let t6 = context.struct_type(&[t2.into()], false);

        assert!(!same_type(&t1.into(), &t2.into()));
        assert!(!same_type(&t3.into(), &t4.into()));
        assert!(!same_type(&t5.into(), &t6.into()));
    }

    #[test]
    fn test_post_traversal_retain_and_skipsubtree() {
        let mut tree = Tree::new();
        let root_id = tree.insert(Node::new(ASTNode::NT(crate::syntax::NT::ExecExp)), id_tree::InsertBehavior::AsRoot).unwrap();
        let node_1 = tree.insert(Node::new(ASTNode::NT(crate::syntax::NT::ExecSentence)), id_tree::InsertBehavior::UnderNode(&root_id)).unwrap();
        let node_1_b1 = tree.insert(Node::new(ASTNode::T(crate::lex::Tokens::AndS)), id_tree::InsertBehavior::UnderNode(&root_id)).unwrap();
        let node_2 = tree.insert(Node::new(ASTNode::NT(crate::syntax::NT::FnBody)), id_tree::InsertBehavior::UnderNode(&node_1)).unwrap();
        let node_2_b1 = tree.insert(Node::new(ASTNode::T(crate::lex::Tokens::And)), id_tree::InsertBehavior::UnderNode(&node_1)).unwrap();
        let mut tree_str = String::new();
        tree.write_formatted(&mut tree_str);
        println!("{}", tree_str);

        let nodes: Vec<&ASTNode> = post_traversal_retain_and_skipsubtree(&tree, &root_id, |x| {
            !matches!(x,
                ASTNode::NT(crate::syntax::NT::ExecExp)
            )
        }, |x| {
            matches!(x,
                ASTNode::NT(crate::syntax::NT::ExecExp)
            )
        }).into_iter().map(|x| tree.get(&x).unwrap().data()).collect();
        let nodes2: Vec<&ASTNode> = post_traversal_retain_and_skipsubtree(&tree, &root_id, |x| {
            !matches!(x,
                ASTNode::T(Tokens::And) |
                ASTNode::T(Tokens::AndS)
            )
        }, |_x| false).into_iter().map(|x| tree.get(&x).unwrap().data()).collect();

        assert_eq!(Some(&&ASTNode::NT(crate::syntax::NT::ExecExp)), nodes.get(0));
        assert_eq!(None, nodes.get(1));
        assert_eq!(Some(&&ASTNode::T(Tokens::And)), nodes2.get(0));
        assert_eq!(Some(&&ASTNode::T(Tokens::AndS)), nodes2.get(1));
        assert_eq!(None, nodes2.get(3));
    }

    #[test]
    /// 测试 skip_func
    fn test_post_traversal_retain_and_skipsubtree_exp() {
        let mut tree = Tree::new();
        let root_id = tree.insert(Node::new(ASTNode::NT(crate::syntax::NT::ExecExp)), id_tree::InsertBehavior::AsRoot).unwrap();
        let node_1 = tree.insert(Node::new(ASTNode::NT(crate::syntax::NT::ExecExpAndS)), id_tree::InsertBehavior::UnderNode(&root_id)).unwrap();
        let node_2_b1 = tree.insert(Node::new(ASTNode::NT(crate::syntax::NT::ExecExpSigOp)), id_tree::InsertBehavior::UnderNode(&node_1)).unwrap();
        let node_2_b2 = tree.insert(Node::new(ASTNode::NT(crate::syntax::NT::ExecR2)), id_tree::InsertBehavior::UnderNode(&node_1)).unwrap();
        let node_3_b1_b1 = tree.insert(Node::new(ASTNode::T(crate::lex::Tokens::Negate)), id_tree::InsertBehavior::UnderNode(&node_2_b1)).unwrap();
        let node_3_b1_b2 = tree.insert(Node::new(ASTNode::T(crate::lex::Tokens::Int(3))), id_tree::InsertBehavior::UnderNode(&node_2_b1)).unwrap();
        let node_4_b2_b1 = tree.insert(Node::new(ASTNode::T(crate::lex::Tokens::AndS)), id_tree::InsertBehavior::UnderNode(&node_2_b2)).unwrap();
        let node_4_b2_b2 = tree.insert(Node::new(ASTNode::NT(crate::syntax::NT::ExecExpSigOp)), id_tree::InsertBehavior::UnderNode(&node_2_b2)).unwrap();
        let node_5_b2_b2_b1 = tree.insert(Node::new(ASTNode::T(Tokens::Negate)), id_tree::InsertBehavior::UnderNode(&node_4_b2_b2)).unwrap();
        let node_5_b2_b2_b2 = tree.insert(Node::new(ASTNode::T(Tokens::Int(4))), id_tree::InsertBehavior::UnderNode(&node_4_b2_b2)).unwrap();
        let mut tree_str = String::new();
        tree.write_formatted(&mut tree_str);
        println!("{}", tree_str);

        let mut nodes: Vec<&ASTNode> = post_traversal_retain_and_skipsubtree(
            &tree,
            &root_id,
            |x| {
                !matches!(x,
                    ASTNode::T(Tokens::OrS) | ASTNode::T(Tokens::AndS) | 
                    ASTNode::T(Tokens::Or) | ASTNode::T(Tokens::And) | 
                    ASTNode::T(Tokens::Eq) | ASTNode::T(Tokens::Ne) | 
                    ASTNode::T(Tokens::Gt) | ASTNode::T(Tokens::Lt) | 
                    ASTNode::T(Tokens::Ge) | ASTNode::T(Tokens::Le) | 
                    ASTNode::T(Tokens::Plus) | ASTNode::T(Tokens::Minus) | 
                    ASTNode::T(Tokens::Mul) | ASTNode::T(Tokens::Div) | 
                    ASTNode::T(Tokens::Mod) | ASTNode::T(Tokens::Negate) | 
                    ASTNode::T(Tokens::Str(_)) | ASTNode::T(Tokens::Int(_)) | 
                    ASTNode::T(Tokens::Decimal(_)) | ASTNode::T(Tokens::Bool(_)) | 
                    ASTNode::NT(NT::ExecMatch) | ASTNode::NT(NT::ExecVar) | 
                    ASTNode::NT(NT::ExecExp)
                )
            },
            |x| {
                // 不对其子树进行遍历
                matches!(
                    x,
                    ASTNode::NT(NT::ExecMatch) | 
                    ASTNode::NT(NT::ExecVar) | 
                    ASTNode::NT(NT::ExecExp)
                )
            },
        ).into_iter().map(|x| tree.get(&x).unwrap().data()).collect();

        assert_eq!(Some(&&ASTNode::NT(NT::ExecExp)), nodes.get(0));
        assert_eq!(None, nodes.get(1));
    }

    #[test]
    fn test1() {
        llvmir_gen_test_macro!("s18.ms", true, i32, Some(0), "main");
    }

    #[test]
    fn test2() {
        llvmir_gen_test_macro!("s19.ms", true, i32, Some(4), "main");
    }

    #[test]
    fn test3() {
        llvmir_gen_test_macro!("s20.ms", true, i32, Some(12), "main");
    }
    
    #[test]
    fn test4() {
        llvmir_gen_test_macro!("s21.ms", true, i32, Some(12), "main");
    }

    #[test]
    fn test5() {
        llvmir_gen_test_macro!("s22.ms", true, i32, Some(14), "main");
    }

    #[test]
    fn test6() {
        llvmir_gen_test_macro!("s23.ms", true, i32, Some(1), "main");
    }

    #[test]
    fn test7() {
        llvmir_gen_test_macro!("s24.ms", false, i32, None::<i32>, "main");
    }

    #[test]
    fn test8() {
        llvmir_gen_test_macro!("s25.ms", true, i32, Some(9), "main");
    }

    #[test]
    fn test9() {
        llvmir_gen_test_macro!("s26.ms", true, i32, Some(11), "main");
    }

    #[test]
    fn test10() {
        llvmir_gen_test_macro!("s27.ms", true, i32, Some(3), "main");
    }
    
    #[test]
    fn test11() {
        llvmir_gen_test_macro!("s28.ms", true, f32, Some(7.754), "main");
    }

    #[test]
    fn test12() {
        llvmir_gen_test_macro!("s29.ms", true, i32, Some(13), "main");
    }
    
    #[test]
    fn test13() {
        llvmir_gen_test_macro!("s30.ms", true, f32, Some(5.3), "main");
    }

    #[test]
    fn test14() {
        llvmir_gen_test_macro!("s31.ms", true, i32, None, "main");
    }

    #[test]
    fn test15() {
        llvmir_gen_test_macro!("s32.ms", true, i32, Some(5), "main");
    }

    #[test]
    fn test16() {
        llvmir_gen_test_macro!("s33.ms", true, i32, Some(6), "main");
    }

    #[test]
    fn test17() {
        llvmir_gen_test_macro!("s34.ms", false, i32, None, "main");
    }

    #[test]
    fn test18() {
        llvmir_gen_test_macro!("s35.ms", false, i32, None, "main");
    }

    #[test]
    fn test19() {
        llvmir_gen_test_macro!("s36.ms", false, i32, None, "main");
    }

    #[test]
    fn test20() {
        llvmir_gen_test_macro!("s37.ms", true, i32, Some(6), "main");
    }
}