//! 编译器入口程序

use std::fmt::Error;
use std::fs::File;
use std::io::Write;

use Merilog::lex::Tokens;
use Merilog::lex::analysis::Analysis;
use Merilog::lex::preprocessor::preprocessor;

pub const VERSION: &str = "0.1.0";
pub const ARGS_PAT: &str = "[-h] [-v] [ -P | -Preprocess ] [ -L | --Lex ] [-o <output>] <input>"; 

fn usage() {
    println!("Merilog use method: {}", ARGS_PAT);
}

fn main() {
    // 获取参数
    let args: Vec<String> = std::env::args().collect();
    // 解析参数
    let mut input = String::new();
    let mut output = String::new();
    let mut just_lex = false;
    let mut just_prepocess = false;
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
                just_lex = true;
            },
            "-P" | "--Preprocess" => {
                just_prepocess = true;
            },
            "-o" => {
                if i + 1 < args.len() {
                    output = args[i + 1].clone();
                    i += 1;
                } else {
                    println!("error: -o <output> need a output file");
                    return;
                }
            }
            _ => {
                if input.is_empty() {
                    input = args[i].clone();
                } else {
                    println!("error: too many input files");
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
        // default
        output = input.clone();
        output.push_str(".out");
    }
    let file =  match std::fs::File::open(&input) {
        Ok(f) => f,
        Err(e) => {
            println!("source file \"{}\" open failed!: {}", &input, e);
            return;
        }
    };
    let mut ft = match File::create(&output) {
        Ok(f) => f,
        Err(e) => {
            println!("can't write compiled code to output \"{}\": {}", &output, e);
            return;
        }
    };
    let raw = preprocessor(&file);
    if just_prepocess {
        ft.write_all(raw.as_bytes()).unwrap();
        return;
    }
    if just_lex {
        let mut analysis = Analysis::new_with_capacity(&input, &raw, raw.len());
        loop {
            let x = match analysis.next_token() {
                Ok(t) => t,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            };
            if x != Tokens::End {
                ft.write_all(format!("{}\n", x.gen_binary_g()).as_bytes()).unwrap();
            } else {
                break;
            }
        }
        return;
    }
}