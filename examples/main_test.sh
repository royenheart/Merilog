# DEBUG
set -x

cargo build

# other info

cargo run -- -h 2>&1 > examples/outs/help
cargo run -- -v 2>&1 > examples/outs/version

# Preprocess & Lex & Syntax & Semantic

cargo run -- examples/sources/s23.ms -o examples/outs/s23.ms.out -P -L -S -I --Visual examples/outs/s23.ms.syntax.svg 2>&1 > examples/outs/s23.ms.compile.log
cargo run -- examples/sources/s25.ms -o examples/outs/s25.ms.out -P -L -S -I --Visual examples/outs/s25.ms.syntax.svg 2>&1 > examples/outs/s25.ms.compile.log
cargo run -- examples/sources/s30.ms -o examples/outs/s30.ms.out -P -L -S -I --Visual examples/outs/s30.ms.syntax.svg 2>&1 > examples/outs/s30.ms.compile.log
cargo run -- examples/sources/s31.ms -o examples/outs/s31.ms.out -P -L -S -I --Visual examples/outs/s31.ms.syntax.svg 2>&1 > examples/outs/s31.ms.compile.log

# Compile to exeutable file with clang(15)

clang-15 examples/outs/s23.ms.out.ll -o examples/outs/s23.ms.main
clang-15 examples/outs/s25.ms.out.ll -o examples/outs/s25.ms.main
clang-15 examples/outs/s30.ms.out.ll -o examples/outs/s30.ms.main
clang-15 examples/outs/s31.ms.out.ll -o examples/outs/s31.ms.main

# Run

examples/outs/s23.ms.main
examples/outs/s25.ms.main
examples/outs/s30.ms.main
examples/outs/s31.ms.main