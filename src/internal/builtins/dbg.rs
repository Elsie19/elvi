use crate::internal::errors::ElviError;
use crate::internal::errors::VariableError;
use crate::internal::status::ReturnCode;
use crate::internal::variables::ElviMutable;
use crate::internal::variables::Variables;

/// The internal code that runs when the `dbg` builtin is run.
#[allow(clippy::module_name_repetitions)]
pub fn builtin_dbg(name: &str, variables: &mut Variables) -> ReturnCode {
    let Some(var) = variables.get_variable(name) else {
        let err = VariableError::NoSuchVariable {
            name: name.to_string(),
            caller: "dbg",
        };
        eprintln!("{err}");
        return err.ret();
    };
    match var.get_modification_status() {
        ElviMutable::Normal => println!("{}={:?}", name, var.get_value()),
        ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
            println!("readonly {}={}", name, var.get_value());
        }
    }
    ReturnCode::SUCCESS.into()
}
