mod parse;

use parse::grammar::{ElviParser, Rule};
use pest::Parser;

fn main() {
    let raw_parse =
        ElviParser::parse(Rule::program, r#"foo="bar\n""#).unwrap_or_else(|e| panic!("{}", e));
    dbg!(&raw_parse);
}
