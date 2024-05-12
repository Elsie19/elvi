mod internal;
mod parse;

use std::fs;

use internal::variables::Variables;
use internal::{commands::Commands, variables::ElviType};
use parse::grammar::{ElviParser, Rule};
// use pest::Parser;
use pest_consume::Parser;

fn main() {
    let unparsed_file = fs::read_to_string("test.elv").expect("Could not read file");

    let mut variables = Variables::default();
    let mut commands = Commands::generate(&variables);

    let raw_parse = ElviParser::parse(Rule::program, &unparsed_file).unwrap();

    let raw_parse = raw_parse.single().unwrap();

    let stuff = ElviParser::program(raw_parse).unwrap();

    for (name, var) in stuff {
        match variables.set_variable(name.to_string(), var) {
            Ok(_) => (),
            Err(oof) => {
                eprintln!("{}", oof);
            }
        }
    }

    dbg!(&variables);
}
