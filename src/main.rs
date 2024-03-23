//! 编译器入口程序

use std::fs::File;
use std::io::Write;
use std::path::Path;

use id_tree_layout::Layouter;
use inkwell::context::Context;
use Merilog::lex::analysis::Analysis;
use Merilog::lex::preprocessor::preprocessor;
use Merilog::semantic::llvmir_gen::IrGen;
use Merilog::syntax::ll_parser::RecursiveDescentParser;
use Merilog::table::symbol::SymbolManager;

pub const VERSION: &str = "0.1.0";
pub const ARGS_PAT: &str = "[-h] [-v] [ -P | -Preprocess ] [ -L | --Lex ] [ -S | -Syntax [--Visual <vi_output>]] [-I | --IR] [-o <output>] <input>";

fn usage() {
    println!("Merilog use method: {}", ARGS_PAT);
}

fn main() {
    // 获取参数
    let args: Vec<String> = std::env::args().collect();
    // 解析参数
    let mut input = String::new();
    let mut output = String::new();
    let mut print_lex = false;
    let mut print_preprocess = false;
    let mut print_syntax = false;
    let mut print_ir = false;
    let mut visualize = false;
    let mut vi_str = String::new();
    let mut version = false;
    let mut help = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" => {
                help = true;
                break;
            }
            "-v" => {
                version = true;
                break;
            }
            "-L" | "--Lex" => {
                print_lex = true;
            }
            "-P" | "--Preprocess" => {
                print_preprocess = true;
            }
            "-S" | "--Syntax" => {
                print_syntax = true;
            }
            "-I" | "--IR" => {
                print_ir = true;
            }
            "--Visual" => {
                if print_syntax {
                    if i + 1 < args.len() {
                        visualize = true;
                        vi_str = args[i + 1].clone();
                        i += 1;
                    } else {
                        println!("error: --Visual <vi_output> need a file to write pic");
                        usage();
                        return;
                    }
                } else {
                    println!("Visualize used only in only syntax mode");
                    usage();
                    return;
                }
            }
            "-o" => {
                if i + 1 < args.len() {
                    output = args[i + 1].clone();
                    i += 1;
                } else {
                    println!("error: -o <output> need a output file");
                    usage();
                    return;
                }
            }
            _ => {
                if input.is_empty() {
                    input = args[i].clone();
                } else {
                    println!("error: too many input files");
                    usage();
                    return;
                }
            }
        }
        i += 1;
    }
    // 处理参数
    if help {
        usage();
        return;
    }
    if version {
        println!("Merilog Compiler {}", VERSION);
        return;
    }
    if input.is_empty() {
        println!("error: no input file");
        return;
    }
    if output.is_empty() {
        output = input.clone();
    }
    let file = match std::fs::File::open(&input) {
        Ok(f) => f,
        Err(e) => {
            println!("source file \"{}\" open failed!: {}", &input, e);
            return;
        }
    };
    let raw = preprocessor(&file);
    let analysis = Analysis::new_with_capacity(&input, &raw, raw.len());
    let mut parser: RecursiveDescentParser = match RecursiveDescentParser::new(analysis) {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    let parse_ok = parser.parse();
    let context = Context::create();
    let mut symbols = SymbolManager::new();
    let mut llvmir_gen_engine = IrGen::new(parser.get_ast(), &context, &mut symbols, "main");
    let result = llvmir_gen_engine.gen();
    if parse_ok {
        println!("语法分析成功！");
    } else {
        println!("语法分析失败！");
    }
    if let Err(e) = &result {
        println!("语义分析失败：{}", e);
        return;
    } else {
        println!("语义分析成功！");
    }
    let verified = llvmir_gen_engine.verify();
    if let Err(e) = &verified {
        println!("IR 代码检查失败：{}", e);
        return;
    } else {
        println!("IR 代码检查成功！");
    }
    if print_preprocess {
        let mut o = output.clone();
        o.push_str(".preprocess");
        let mut ft_preprocess = match File::create(&o) {
            Ok(f) => f,
            Err(e) => {
                println!("无法创建预处理结果文件 \"{}\": {}", &o, e);
                return;
            }
        };
        ft_preprocess.write_all(raw.as_bytes()).unwrap();
    }
    if print_lex {
        let mut o = output.clone();
        o.push_str(".lex");
        let mut ft_lex = match File::create(&o) {
            Ok(f) => f,
            Err(e) => {
                println!("无法创建词法分析结果文件 \"{}\": {}", &o, e);
                return;
            }
        };
        parser.get_tokens().clone().into_iter().for_each(|x| {
            ft_lex
                .write_all(format!("{}\n", x.dump()).as_bytes())
                .unwrap()
        });
    }
    if print_syntax {
        let mut o = output.clone();
        o.push_str(".syntax");
        let mut ft_syntax = match File::create(&o) {
            Ok(f) => f,
            Err(e) => {
                println!("无法创建语法分析结果文件 \"{}\": {}", &o, e);
                return;
            }
        };
        match parse_ok {
            true => {
                ft_syntax
                    .write_all(format!("{}\n", parser.dump()).as_bytes())
                    .unwrap();
                if visualize {
                    let t = parser.get_ast();
                    Layouter::new(t)
                        .with_file_path(Path::new(&vi_str))
                        .write()
                        .expect("Failed to write to file");
                }
            }
            false => {
                println!("Syntax analysis failed!");
            }
        }
    }
    if print_ir {
        let mut o = output;
        o.push_str(".ll");
        match File::create(&o) {
            Ok(_) => (),
            Err(e) => {
                println!("无法创建语义分析结果文件 \"{}\": {}", &o, e);
                return;
            }
        };
        llvmir_gen_engine.dump_to_file(o);
    }
}
