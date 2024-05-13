mod internal;
mod parse;

use std::fs;

use internal::commands::Commands;
use internal::variables::Variables;
use parse::grammar::{ElviParser, Rule};
// use pest::Parser;
use pest_consume::Parser;

fn main() {
    let unparsed_file = fs::read_to_string("test.elv").expect("Could not read file");

    pest::set_error_detail(true);

    let mut variables = Variables::default();
    let mut commands = Commands::generate(&variables);

    let raw_parse = match ElviParser::parse_with_userdata(
        Rule::program,
        &unparsed_file,
        (&mut variables, &mut commands),
    ) {
        Ok(yay) => yay,
        Err(oof) => {
            eprintln!("Error: {}", oof.to_string());
            std::process::exit(1);
        }
    };

    let raw_parse = raw_parse.single().unwrap();

    let stuff = match ElviParser::program(raw_parse) {
        Ok(yay) => yay,
        Err(oof) => {
            eprintln!("Error: {}", oof.to_string());
            std::process::exit(1);
        }
    };
}
