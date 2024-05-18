use crate::internal::commands::Commands;
use crate::internal::status::ReturnCode;
use crate::internal::variables::Variables;

/// The internal code that runs when the `hash` builtin is run.
pub fn builtin_hash(
    flag: Option<String>,
    commands: &mut Commands,
    variables: &Variables,
) -> ReturnCode {
    if flag.is_some() {
        if flag.unwrap() == "-r" {
            *commands = Commands::generate(&variables);
        }
    } else {
        for (cmd, patho) in commands.clone().into_iter() {
            println!("{}={}", cmd, patho.into_os_string().into_string().unwrap());
        }
    }
    ReturnCode::ret(ReturnCode::SUCCESS)
}
