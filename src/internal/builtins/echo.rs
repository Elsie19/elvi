use crate::internal::status::ReturnCode;
use crate::internal::variables::{ElviType, Variables};

/// The internal code that runs when the `echo` builtin is run.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn builtin_echo(text: Option<&[ElviType]>, variables: &Variables) -> ReturnCode {
    let mut to_print = vec![];
    if let Some(text) = text {
        for part in text {
            to_print.push(
                part.tilde_expansion(variables)
                    .eval_escapes()
                    .eval_variables(variables)
                    .to_string(),
            );
        }
        println!("{}", to_print.join(" "));
        ReturnCode::SUCCESS.into()
    } else {
        println!();
        ReturnCode::SUCCESS.into()
    }
}
