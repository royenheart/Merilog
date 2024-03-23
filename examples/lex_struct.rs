//! 词法分析器总体结构设计

use std::env;

use graphviz_rust::cmd::CommandArg;
use graphviz_rust::cmd::Format;
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::exec;
use graphviz_rust::printer::PrinterContext;

fn main() {
    // 总体结构设计生成
    let s = graph!(strict di id!("s");
        node!("m"; attr!("label", "mistake"), attr!("shape", "record"), attr!("color", "green")),
        node!("p"; attr!("label", "preprocess"), attr!("shape", "record"), attr!("color", "red")),
        node!("l"; attr!("label", "lex"), attr!("shape", "record"), attr!("color", "blue")),
        node!("b"; attr!("label", "buffer"), attr!("shape", "ellipse")),
        node!("s"; attr!("label", "scan"), attr!("shape", "plaintext")),
        edge!(node_id!("p") => node_id!("b") => node_id!("l"); attr!("arrowhead", "halfopen")),
        edge!(node_id!("p") => node_id!("m"); attr!("arrowhead", "none"), attr!("rank", "same")),
        edge!(node_id!("l") => node_id!("m"); attr!("arrowhead", "none")),
        edge!(node_id!("l") => node_id!("s"); attr!("arrowhead", "obox"), attr!("label", "method"))
    );
    let mut path = env::current_dir().unwrap();
    path.push("examples/tests/lex_struct.png");
    let mut ctx = PrinterContext::default();
    ctx.always_inline();
    exec(
        s,
        &mut ctx,
        vec![
            CommandArg::Format(Format::Png),
            CommandArg::Output(path.to_str().unwrap().to_string()),
        ],
    )
    .unwrap();
}
