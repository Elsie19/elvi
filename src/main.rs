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
//!
//! # Notes
//! 1. Elvi is *not* a shell to be used for interactive purposes. It's sole purpose is to be a
//!    command-line interpreter.

pub mod internal;
pub mod parse;
pub mod user_flags;

use std::fs;

use clap::{error::ErrorKind, CommandFactory, Parser as ClapParser};
use parse::grammar::{ElviParser, Rule};
use pest_consume::Parser;
use user_flags::Args;

#[doc(hidden)]
fn main() {
    let args = Args::parse();
    let unparsed_file = if let Some(input) = args.group.read_from_input {
        input
    } else {
        match fs::read_to_string(&args.group.file.as_ref().unwrap()) {
            Ok(yay) => yay,
            Err(_) => Args::command()
                .error(
                    ErrorKind::ValueValidation,
                    format!(
                        "File `{}` does not exist.",
                        args.group.file.unwrap().to_str().unwrap()
                    ),
                )
                .exit(),
        }
    };

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
