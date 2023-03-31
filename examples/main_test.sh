# DEBUG
set -x

cargo build

# Lex

cargo run -- -h 2>&1 > examples/outs/main_help
cargo run -- -v 2>&1 > examples/outs/main_version
cargo run -- examples/sources/s1.ms -o examples/outs/s1_pre -P 2>&1 > examples/outs/main_preprocess_s1
cargo run -- examples/sources/s1.ms -o examples/outs/s1_lex -L 2>&1 > examples/outs/main_lex_s1
cargo run -- examples/sources/s7.ms -o examples/outs/s7_lex -L 2>&1 > examples/outs/main_lex_s7

# Syntax

cargo run -- examples/sources/s8.ms -o examples/outs/s8_syntax -S --Visual examples/outs/s8_syntax.svg 2>&1 > examples/outs/main_syntax_s8
cargo run -- examples/sources/s9.ms -o examples/outs/s9_syntax -S 2>&1 > examples/outs/main_syntax_s9
cargo run -- examples/sources/s10.ms -o examples/outs/s10_syntax -S 2>&1 > examples/outs/main_syntax_s10
cargo run -- examples/sources/s11.ms -o examples/outs/s11_syntax -S 2>&1 > examples/outs/main_syntax_s11
cargo run -- examples/sources/s12.ms -o examples/outs/s12_syntax -S 2>&1 > examples/outs/main_syntax_s12
cargo run -- examples/sources/s13.ms -o examples/outs/s13_syntax -S 2>&1 > examples/outs/main_syntax_s13
cargo run -- examples/sources/s14.ms -o examples/outs/s14_syntax -S 2>&1 > examples/outs/main_syntax_s14