# DEBUG
set -x

cargo build
cargo run -- -h 2>&1 > examples/outs/main_help
cargo run -- -v 2>&1 > examples/outs/main_version
cargo run -- examples/sources/s1.ms -o examples/outs/s1_pre -P 2>&1 > examples/outs/main_preprocess_s1
cargo run -- examples/sources/s1.ms -o examples/outs/s1_lex -L 2>&1 > examples/outs/main_lex_s1
cargo run -- examples/sources/s7.ms -o examples/outs/s7_lex -L 2>&1 > examples/outs/main_lex_s7