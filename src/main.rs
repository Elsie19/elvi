mod internal;
mod parse;

use std::fs;

use parse::grammar::{ElviParser, Rule};
use pest_consume::Parser;

fn main() {
    let unparsed_file = fs::read_to_string("test.elv").expect("Could not read file");

    let raw_parse = match ElviParser::parse(Rule::program, &unparsed_file) {
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
