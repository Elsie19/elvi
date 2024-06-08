use crate::internal::status::ReturnCode;
use crate::internal::variables::{ElviType, Variables};

/// The internal code that runs when the `echo` builtin is run.
pub fn builtin_echo(text: Option<Vec<ElviType>>, variables: &Variables) -> ReturnCode {
    let mut to_print = vec![];
    if text.is_none() {
        println!();
        return ReturnCode::ret(ReturnCode::SUCCESS);
    }
    for part in text.unwrap() {
        to_print.push(
            part.tilde_expansion(variables)
                .eval_escapes()
                .eval_variables(variables)
                .to_string(),
        );
    }
    println!("{}", to_print.join(" "));
    ReturnCode::ret(ReturnCode::SUCCESS)
}
