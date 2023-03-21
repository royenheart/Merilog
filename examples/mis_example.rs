use Merilog::mistakes::show::LineType;
use Merilog::mistakes::show::Mis;
use Merilog::mistakes::show::Froms;
use Merilog::mistakes::show::Types;

fn main() {
    let mut e = Mis::new(Froms::Lex, Types::Error, "未知符号", "test.ms", None);
    e.add_line(0, "出现未知符号，词法分析无法识别", "fn main(]", Some(LineType::Happen), Some((9, 1)));
    println!("{}", e);
}