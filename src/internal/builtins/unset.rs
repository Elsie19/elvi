use crate::internal::status::ReturnCode;
use crate::internal::variables::ElviMutable;
use crate::internal::variables::VariableError;
use crate::internal::variables::Variables;

/// The internal code that runs when the `unset` builtin is run.
pub fn builtin_unset(name: &str, variables: &mut Variables) -> ReturnCode {
    let Some(var) = variables.get_variable(name) else {
        // <https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/utilities/V3_chap02.html#unset> in description in 5th paragraph
        return ReturnCode::ret(ReturnCode::SUCCESS);
    };
    match var.get_modification_status() {
        ElviMutable::Normal => match variables.unset(name) {
            // We don't care about what it returned
            Some(_) | None => return ReturnCode::ret(ReturnCode::SUCCESS),
        },
        ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
            eprintln!(
                "{}",
                VariableError::Readonly {
                    name: "unset".to_string(),
                    line: var.get_line().0,
                    column: var.get_line().1
                }
            );
            return ReturnCode::ret(ReturnCode::MISUSE);
        }
    }
}
