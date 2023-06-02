use inkwell::{context::Context, module::Module, AddressSpace, values::{PointerValue, AnyValueEnum, ArrayValue, FunctionValue, BasicValue, AnyValue}, builder::Builder, types::{VoidType, IntType, AnyTypeEnum, ArrayType, BasicMetadataTypeEnum}};

fn main() {
    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let gen = test_llvm_ir_example::new(&context, &builder, &module);
    gen.ir_gen();
    gen.jit_execute();
}

struct test_llvm_ir_example<'a, 'ctx> {
    context: &'ctx Context,
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
}

impl<'a, 'ctx> test_llvm_ir_example<'a, 'ctx> {
    fn new(context: &'ctx Context, builder: &'a Builder<'ctx>, module: &'a Module<'ctx>) -> test_llvm_ir_example<'a, 'ctx> {
        test_llvm_ir_example {
            context, builder, module
        }
    }

    fn ir_gen(&self) {
        // let i32_type = self.context.i32_type();
        // let str_type = self.context.i8_type().ptr_type(AddressSpace::default());
        // let printf_type = i32_type.fn_type(&[str_type.into()], true);
        // let printf_func = self.module.add_function("puts", printf_type, Some(inkwell::module::Linkage::External));

        let i32_type = self.context.i32_type();
        let func1_type = i32_type.fn_type(&[i32_type.into()], false);
        // let arr_t = i32_type.array_type(4);
        // let arr_t_p = arr_t.ptr_type(AddressSpace::default());
        // let func2_type = i32_type.fn_type(&[arr_t_p.into()], false);

        let func1 = self.module.add_function("test_struct", func1_type, None);
        // let func2 = self.module.add_function("fun2", func2_type, None);
        // let block1 = self.context.append_basic_block(func1, "test_struct");
        
        // self.builder.position_at_end(block1);
        // self.emit_printf_call("Hello World\n", "block1print", &printf_func);
        // self.emit_struct();
        
        // let block2 = self.context.append_basic_block(func1, "2");
        // let c = self.builder.build_unconditional_branch(block2);
        // self.builder.position_at_end(block2);
        // self.emit_printf_call("World Deny\n", "block2print", &printf_func);

        // let i1 = self.context.i16_type().const_int(20, false);
        // let i2 = self.context.i16_type().const_int(30, false);
        // let i3 = self.builder.build_int_add(i1, i2, "add");
        // let i4 = self.builder.build_int_add(i3, i1, "add");
        // let ii = self.builder.build_alloca(self.context.i16_type(), "ii");
        // let il = self.builder.build_alloca(self.context.i128_type(), "ii");
        // self.builder.build_store(il, i4);
        // let tmp = self.builder.build_alloca(self.context.i16_type(), "tmp");
        // self.builder.build_store(ii, i4);
        // self.builder.build_store(tmp, i4);
        // let d = self.context.struct_type(&[], false).const_named_struct(&[]);

        // println!("{:?}", ii.get_type());
        // let ii_v = self.builder.build_load(self.context.i16_type(), ii, "ii_v");
        // println!("{:?}", ii_v);
        // println!("{:?}", ii_v.into_int_value());
        // println!("{:?}", ii.as_basic_value_enum());
        // println!("{:?}", tmp.get_type());
        // println!("{:?}", tmp.get_name());
        // let tmp_v = self.builder.build_load(self.context.i16_type(), tmp, "loadtmp");
        // let i8 = self.builder.build_not(tmp_v.into_int_value(), "not");
        // // let i5 = self.builder.build_int_add(ii_v.into_int_value(), i4, "add");
        // self.builder.build_store(ii, i5);

        // let fmt_str = self.context.const_string("%d\n".as_bytes(), false);
        // let fmt_str_ptr = self.builder.build_alloca(fmt_str.get_type(), "fmt_str");
        // self.builder.build_store(fmt_str_ptr, fmt_str);
        // let fmt_str_ptr_cast = self.builder.build_pointer_cast(fmt_str_ptr, str_type, "fmt_str_ptr");

        // let istr = self.builder.build_int_to_ptr(i8, str_type, "i82str");
        // // let istr = self.builder.build_pointer_cast(, str_type, "int2str");
        // self.builder.build_call(printf_func, &[istr.into()], "i42ASCIIprint");
        // self.builder.build_call(printf_func, &[fmt_str_ptr_cast.into(), i8.into()], "i8print");

        // let i1_type = self.context.bool_type();
        // let l = self.context.bool_type().const_int(3, false);
        // let r = self.context.bool_type().const_int(0, false);
        // let true_block = self.context.append_basic_block(func1, "true_block");
        // let false_block = self.context.append_basic_block(func1, "false_block");
        // let merge_block = self.context.append_basic_block(func1, "merge_block");
        // self.builder.build_conditional_branch(l, true_block, false_block);

        // self.builder.build_array_alloca(ty, size, name)
        // self.builder.position_at_end(true_block);
        // let true_result = i1_type.const_int(1, false);
        // self.builder.build_unconditional_branch(merge_block);

        // self.builder.position_at_end(false_block);
        // let false_result = r;
        // self.builder.build_unconditional_branch(merge_block);

        // self.builder.position_before(&c);
        // self.emit_printf_call("Test Switch\n", "block1print0", &printf_func);

        // // phi 基本块内传值
        // self.builder.position_at_end(merge_block);
        // let result = self.builder.build_phi(i1_type, "or_result");
        // result.add_incoming(&[(&true_result, true_block), (&false_result, false_block)]);

        // let arrr = self.context.i32_type().array_type(4).const_zero();    
        // println!("{:?}", arrr.get_name());
        // println!("{:?}", arrr.get_type().get_element_type());

        // println!("{:?}", result);
        // println!("{:?}", result.print_to_string());
        // println!("{:?}", result.as_any_value_enum());
        // // println!("{:?}", istr.as_any_value_enum());
        // println!("{:?}", func1.get_first_param().unwrap());
        // println!("{:?}", func1.get_first_param().unwrap().as_any_value_enum());
        // println!("{:?}", func1.as_any_value_enum());

        // 创建一个 i32 类型的全局变量
        // let global_var = self.module.add_global(self.context.i32_type(), Some(AddressSpace::default()), "my_global");
        // global_var.set_initializer(&self.context.i32_type().const_int(42, false));
        // // 创建一个不可变指针
        // let immutable_ptr = global_var.as_pointer_value();
        // let immutable_value = self.builder.build_load(self.context.i32_type(), immutable_ptr, "immutable_value");

        // let new_immutable_value = self.builder.build_int_add(immutable_value.into_int_value(), self.context.i32_type().const_int(10, false), "new_immutable_value");
        // self.builder.build_store(immutable_ptr, new_immutable_value);

        // 创建一个不可变指针
        // let immutable_ptr = global_var.as_pointer_value();

        // let i32_const = self.context.i32_type().const_int(15, false);
        // let arr_t_var = self.builder.build_alloca(arr_t, "init_arr_t");
        // let i1 = i32_type.const_int(0, false);
        // let i2 = i32_type.const_int(1, false);
        // let i3 = i32_type.const_int(2, false);
        // let i4 = i32_type.const_int(3, false);
        // let arr_t_var_ptr1 = unsafe { self.builder.build_gep(i32_type, arr_t_var, &[i1], "load_arr_t_0") };
        // let arr_t_var_ptr2 = unsafe { self.builder.build_gep(i32_type, arr_t_var, &[i2], "load_arr_t_1") };
        // let arr_t_var_ptr3 = unsafe { self.builder.build_gep(i32_type, arr_t_var, &[i3], "load_arr_t_2") };
        // let arr_t_var_ptr4 = unsafe { self.builder.build_gep(i32_type, arr_t_var, &[i4], "load_arr_t_3") };
        // self.builder.build_store(arr_t_var_ptr1, i1);
        // self.builder.build_store(arr_t_var_ptr2, i2);
        // self.builder.build_store(arr_t_var_ptr3, i3);
        // self.builder.build_store(arr_t_var_ptr4, i4);
        // println!("{:?}", arr_t_var.print_to_string());
        // let func_2_call1 = self.builder.build_call(func2, &[arr_t_var.into()], "func_2_call_1");

        // for i in func1.get_param_iter() {
        //     println!("func1 params value: {:?}", i);
        //     println!("func1 params type: {:?}", i.get_type());
        //     println!("func1 params v: {:?}", i.print_to_string());
        //     self.builder.build_int_add(i.into_int_value(), i32_const, "add");
        // }

        // for i in func2.get_param_iter() {
        //     println!("func2 params value: {:?}", i);
        //     println!("func2 params type: {:?}", i.get_type());
        //     println!("func2 params v: {:?}", i.print_to_string())
        // }

        // for i in func_2_call1.get_called_fn_value().get_param_iter() {
        //     println!("func2 call1 params value: {:?}", i);
        //     println!("func2 call1 params type: {:?}", i.get_type());
        //     println!("func2 call1 params v: {:?}", i.print_to_string())
        // }

        // self.builder.build_return(Some(&i32_type.const_int(0, false)));
    
        // let func2_b = self.context.append_basic_block(func2, "func2_b");
        // self.builder.position_at_end(func2_b);
        // 从符号表中获取 Type ，然后可以得到
        // Value 是 PointerValue，类型是 PointerValue 指向的 ArrayType

        // 1. 获取原符号 PointValue（符号表或者上一次迭代）；若为 FuncValue，调用，创建临时指针指向返回值；若是右值，即不是变量 PointValue 只是值，也新建一个临时指针指向以及类型，此时不管后续如何都无法返回引用，只能是值
        // let p = func2.get_nth_param(0).unwrap().into_pointer_value();
        // // 2. 获取原符号 PointValue 的值，根据符号表中提供的类型或者上一步得到的类型（原符号类型）转换成对应 Value；
        // let pi = self.builder.build_load(arr_t, p, "load_func2_0").into_array_value();
        // 3. 根据获取的值以及引用符号，判断出内部元素的类型；或进行函数调用，函数调用后，将只存在右值（非 PointerValue 即指针，函数返回值），
        // 先重新构造一个临时指针（变量）指向函数调用的值，下一步的 PointerValue 即为该临时指针，类型即为函数的返回值的类型，跳到第 1 步。；
        // 如果设置过右值，需要返回，只返回值，不返回引用
        // let pi_pt = pi.get_type().get_element_type();
        // // 4. 根据内部元素类型，加上原符号 PointerValue，可以获取到内部元素的 PointerValue 即指向
        // let pi_0 = unsafe { self.builder.build_gep(pi_pt, p, &[i1], "load_func2_0_0") };
        // 可以看到，我们已经获取到了下一步符号的 PointerValue（内部元素 PointerValue 指向），下一步符号的类型（内部元素的类型），可以进行迭代了
        // 在第 3 步，便可以根据此时的引用符号和原符号类型看是否合法引用，非法引用将无法继续
        // 在第 3 步，若此时没有其他引用符号，若进行过函数调用或者当前就是 FuncValue（当前是就先调用一次，得到返回值替换当前原符号 PointValue 的值），只返回原符号 PointValue 的值，无引用；若没有函数调用过且当前不是 FuncValue，返回当前的原符号的 PointValue 和 原符号 PointValue 的值。
        // 最后还需要根据 vt 和 vp 获取一下 

        // self.emit_printf_call(&format!("Iam Func2, receive pi: {:?}\nreceive pi_0: {:?}", pi.print_to_string(), pi_0.print_to_string()), "func2callprint", &printf_func);
        // self.builder.build_return(Some(&i32_type.const_int(0, false)));

        let l = self.context.bool_type().const_zero();
        let r = self.context.bool_type().const_zero();
        let i1_type: IntType = self.context.bool_type();
        let true_block = self
            .context
            .append_basic_block(func1, "true_block");
        let false_block = self
            .context
            .append_basic_block(func1, "false_block");
        let merge_block = self
            .context
            .append_basic_block(func1, "merge_block");
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
        println!("{:?}", ret);

        println!("{}", self.module.print_to_string().to_string());
    }

    fn jit_execute(&self) {
        match (self.module.verify(), self.module.create_jit_execution_engine(inkwell::OptimizationLevel::None)) {
            (Ok(_), Ok(engine)) => {
                if let Ok(f) = unsafe {
                    engine.get_function::<unsafe extern "C" fn() -> i32>("test_struct")
                } {
                    unsafe {
                        f.call();
                    }
                }
            },
            (Err(x), _) => {
                println!("{}", x);
            },
            (_, Err(x)) => {
                println!("{}", x);
            }
        }
    }

    fn test_ptr_value(&self) {
        let i32_type = self.context.i32_type();
        let i1 = i32_type.const_int(0, false);
        let i2 = i32_type.const_int(1, false);
        let i32_v1 = i32_type.const_int(0, false);
        let i32_v2 = i32_type.const_int(1, false);
        let struct_type = self.context.struct_type(&[i32_type.into()], false);
        let struct_v1 = struct_type.const_named_struct(&[i32_v1.into(), i32_v2.into()]);
        let struct_v2 = struct_type.const_named_struct(&[i32_v2.into(), i32_v1.into()]);
        let arr_type = struct_type.array_type(2);
        // arr_v 存储在符号表中（使用 build_alloca 创建的变量，一定是 PointerValue）
        let mut arr_v = self.builder.build_alloca(arr_type, "arr_v_var");
        self.builder.build_store(arr_v, arr_type.const_zero());
        let arr_v = arr_v.as_any_value_enum();
        // 模仿引用解析，首先需要针对变量，获取引用
        let vp = arr_v.into_pointer_value();
        // println!("{:?}", vp.get_type());
        
        // 接下来解析数组，首先获取值
        let vv = self.builder.build_load(arr_type, vp, "get_arr_v_point_value");
        // 对于数组，将其
        
        // let mut arr_i_ptr1 = unsafe { self.builder.build_gep(struct_type, arr_v_e, &[i1.into()], "load_arr_1_ptr") };
        // let mut arr_i_ptr2 = unsafe { self.builder.build_gep(struct_type, arr_v_e, &[i2.into()], "load_arr_2_ptr") };
        // let mut arr_i_v1 = self.builder.build_load(, ptr, name)
    }

    fn emit_printf_call(&self, hello_str: &str, name: &str, func: &FunctionValue) -> IntType {
        // let pointer_value = self.emit_global_string(hello_str, name);
        // let hstr = self.context.const_string(hello_str.as_bytes(), false);
        let hstr_ptr = self.builder.build_global_string_ptr(hello_str, &name);
        let c = self.builder.build_call(*func, &[hstr_ptr.as_pointer_value().into()], name);
        // println!("{:?}", c.as_any_value_enum());
        // println!("{:?}", c.get_called_fn_value());
 
        self.context.i32_type()
    }

    fn emit_struct(&self) {
        let s = self.context.struct_type(&[
            self.context.i32_type().into()
        ], false);
        let v = self.module.add_global(s, Some(AddressSpace::default()), "test_struct");
        v.set_initializer(&s.const_zero());
        v.set_linkage(inkwell::module::Linkage::Internal);
        // println!("struct any value enum: {:?}", v.as_any_value_enum());

        // let sptr = self.builder.build_alloca(s, "struct_mem1");
        // self.builder.build_store(sptr, v.as_pointer_value());
    }

    fn emit_global_string(&self, strs: &str, name: &str) -> PointerValue<'a> {
        let ty = self.context.i8_type().array_type(strs.len() as u32);
        let gv = self.module.add_global(ty, Some(AddressSpace::default()), name);
        gv.set_linkage(inkwell::module::Linkage::Internal);
        gv.set_initializer(&self.context.const_string(strs.as_ref(), false));

        let pointer_value = self.builder.build_pointer_cast(
            gv.as_pointer_value(),
            self.context.i8_type().ptr_type(AddressSpace::default()),
            name
        );

        pointer_value
    }
}