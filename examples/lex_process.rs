//! 词法分析器流程和关键算法

use std::env;

use graphviz_rust::cmd::CommandArg;
use graphviz_rust::cmd::Format;
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::exec;
use graphviz_rust::printer::PrinterContext;

fn main() {
    // 词法分析流程
    let f = graph!(strict di id!("f");
        node!("p"; attr!("label", "preprocess"), attr!("shape", "record"), attr!("color", "red")),
        node!("l"; attr!("label", "lex"), attr!("shape", "record"), attr!("color", "blue")),
        node!("b"; attr!("label", "buffer"), attr!("shape", "ellipse")),
        node!("s"; attr!("label", "scan"), attr!("shape", "plaintext")),
        edge!(node_id!("p") => node_id!("b") => node_id!("l") => node_id!("s"); attr!("arrowhead", "halfopen"))
    );
    let mut path = env::current_dir().unwrap();
    path.push("examples/tests/lex_flow.png");
    let mut ctx = PrinterContext::default();
    ctx.always_inline();
    exec(
        f,
        &mut ctx,
        vec![
            CommandArg::Format(Format::Png),
            CommandArg::Output(path.to_str().unwrap().to_string()),
        ],
    )
    .unwrap();
}
