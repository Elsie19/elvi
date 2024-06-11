use crate::internal::commands::Commands;
use crate::internal::errors::{CommandError, ElviError};
use crate::internal::status::ReturnCode;
use crate::internal::variables::{ElviType, Variables};

/// The internal code that runs when the `hash` builtin is run.
#[allow(clippy::module_name_repetitions)]
pub fn builtin_hash(
    flag: Option<ElviType>,
    commands: &mut Commands,
    variables: &Variables,
) -> ReturnCode {
    if flag.is_some() {
        if *flag.as_ref().unwrap() == ElviType::String("-r".into()) {
            *commands = Commands::generate(variables);
        } else {
            let err = CommandError::SubCommandNotFound {
                name: "hash",
                cmd: flag.unwrap().to_string(),
            };
            eprintln!("{err}");
            return err.ret();
        }
    } else {
        for (cmd, patho) in &commands.cmds {
            println!("{}={}", cmd, patho.display());
        }
    }
    ReturnCode::SUCCESS.into()
}
