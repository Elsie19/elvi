//! A POSIX shell written in Rust.

pub mod internal;
pub mod parse;

use std::fs;

use parse::grammar::{ElviParser, Rule};
use pest_consume::Parser;

#[doc(hidden)]
fn main() {
    let unparsed_file = fs::read_to_string("test.elv").expect("Could not read file");

    let raw_parse = match ElviParser::parse(Rule::program, &unparsed_file) {
        Ok(yay) => yay,
        Err(oof) => {
            eprintln!("{oof}");
            std::process::exit(1);
        }
    };

    let raw_parse = raw_parse.single().unwrap();

    std::process::exit(ElviParser::program(raw_parse).get().into());
}
