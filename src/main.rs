mod internal;
mod parse;

use std::fs;

use internal::commands::Commands;
use internal::variables::Variables;
use parse::grammar::{ElviParser, Rule};
use pest::Parser;

fn main() {
    let unparsed_file = fs::read_to_string("test.elv").expect("Could not read file");

    let mut variables = Variables::default();
    let mut commands = Commands::generate(&variables);

    let raw_parse =
        ElviParser::parse(Rule::program, &unparsed_file).unwrap_or_else(|e| panic!("{}", e));
    dbg!(variables);
    dbg!(commands);
    dbg!(&raw_parse);
}
