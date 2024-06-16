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

use std::{env, fs};

use clap::{error::ErrorKind, CommandFactory, Parser as ClapParser};
use internal::{status::ReturnCode, variables::Arguments};
use parse::grammar::{ElviParser, Rule};
use pest_consume::Parser;
use user_flags::Args;

#[doc(hidden)]
fn main() {
    let args = Args::parse();
    let unparsed_file = if let Some(ref input) = args.group.read_from_input {
        input
    } else {
        &match fs::read_to_string(args.group.file.as_ref().unwrap()) {
            Ok(yay) => yay,
            Err(_) => Args::command()
                .error(
                    ErrorKind::ValueValidation,
                    format!(
                        "File `{}` does not exist.",
                        args.group.file.unwrap().display()
                    ),
                )
                .exit(),
        }
    };

    let file = if let Some(ref v) = args.group.file {
        v.to_str().unwrap()
    } else {
        "stdin"
    };

    let var_zero = if args.group.read_from_input.is_some() {
        env::current_exe().unwrap().to_str().unwrap().to_string()
    } else {
        args.group
            .file
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    };

    let mut positionals: Arguments = vec![var_zero].into();

    if let Some(mut positional_args) = args.positionals {
        positionals.args.append(&mut positional_args);
    }

    // Check if we can successfully parse the script on first go.
    let raw_parse =
        match ElviParser::parse_with_userdata(Rule::program, unparsed_file, &positionals) {
            Ok(yay) => yay,
            Err(oof) => {
                eprintln!("{}", oof.with_path(file));
                std::process::exit(ReturnCode::MISUSE.into());
            }
        };

    // Get the only top-level `statement`.
    let raw_parse = raw_parse.single().unwrap();

    // Run it.
    std::process::exit(ElviParser::program(raw_parse).get().into());
}
