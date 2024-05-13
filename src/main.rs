mod internal;
mod parse;

use std::fs;
use std::process::Command;

use internal::variables::Variables;
use internal::{commands::Commands, variables::ElviType};
use parse::grammar::{ElviParser, Rule};
// use pest::Parser;
use pest_consume::Parser;

use crate::internal::tree::{Actions, Builtins};

fn main() {
    let unparsed_file = fs::read_to_string("test.elv").expect("Could not read file");

    pest::set_error_detail(true);

    let mut variables = Variables::default();
    let mut commands = Commands::generate(&variables);

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

    for part in stuff.exprs {
        match part {
            Actions::ChangeVariable((name, var)) => match variables.set_variable(name, var) {
                Ok(_) => (),
                Err(oof) => eprintln!("{oof}"),
            },
            Actions::Builtin(built) => match built {
                Builtins::Dbg(text) => println!(
                    "Variable: {} | Contents: {:?}",
                    text,
                    variables.get_variable(&text)
                ),
            },
            Actions::Command(cmd) => {
                let cmd_path = match commands.get_path(cmd.get(0).unwrap()) {
                    Some(yes) => yes,
                    None => {
                        eprintln!("{}: command not found", cmd.get(0).unwrap());
                        continue;
                    }
                };
                Command::new(cmd_path);
                ()
            }
            Actions::Null => (),
        }
    }
}
