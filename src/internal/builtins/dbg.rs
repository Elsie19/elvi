use crate::internal::errors::VariableError;
use crate::internal::status::ReturnCode;
use crate::internal::variables::ElviMutable;
use crate::internal::variables::Variables;

/// The internal code that runs when the `dbg` builtin is run.
pub fn builtin_dbg(name: &str, variables: &mut Variables) -> ReturnCode {
    let Some(var) = variables.get_variable(name) else {
        eprintln!(
            "{}",
            VariableError::NoSuchVariable {
                name: name.to_string(),
                caller: "dbg"
            }
        );
        return ReturnCode::ret(ReturnCode::FAILURE);
    };
    match var.get_modification_status() {
        ElviMutable::Normal => println!("{}={:?}", name, var.get_value()),
        ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
            println!("readonly {}={}", name, var.get_value());
        }
    }
    ReturnCode::ret(ReturnCode::SUCCESS)
}
