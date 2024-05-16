//! A POSIX shell written in Rust.
//!
//! # Getting Started
//! To get started using Elvi, just install the binary and run any POSIX shell script!
//!
//! # Hacking on Elvi
//! ## Creating a builtin
//! To get started, you can check out how to implement your own builtin in [`internal::builtins`],
//! which should offer a complete guide.
//! ## Modifying grammar
//! To modify grammar to add your own non-POSIX creation, check out `src/parse/internals/` and
//! make sure to look at [`pest_consume`] and [`pest_derive`] to understand how to both parse and
//! create grammar!

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
