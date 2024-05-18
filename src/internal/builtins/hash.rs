use crate::internal::commands::{CommandError, Commands};
use crate::internal::status::ReturnCode;
use crate::internal::variables::{ElviType, Variables};

/// The internal code that runs when the `hash` builtin is run.
pub fn builtin_hash(
    flag: Option<ElviType>,
    commands: &mut Commands,
    variables: &Variables,
) -> ReturnCode {
    if flag.is_some() {
        if *flag.as_ref().unwrap() == ElviType::String("-r".into()) {
            *commands = Commands::generate(variables);
        } else {
            eprintln!(
                "{}",
                CommandError::SubCommandNotFound {
                    name: "hash".to_string(),
                    cmd: flag.unwrap().to_string(),
                }
            );
            return ReturnCode::ret(ReturnCode::FAILURE);
        }
    } else {
        for (cmd, patho) in commands.clone().into_iter() {
            println!("{}={}", cmd, patho.into_os_string().into_string().unwrap());
        }
    }
    ReturnCode::ret(ReturnCode::SUCCESS)
}
