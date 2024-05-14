use crate::internal::status::ReturnCode;
use crate::internal::variables::ElviMutable;
use crate::internal::variables::Variables;

pub fn builtin_dbg(name: &str, variables: &mut Variables) -> ReturnCode {
    let Some(var) = variables.get_variable(name) else {
        eprintln!("dbg: no such variable: {name}");
        return ReturnCode::ret(ReturnCode::FAILURE);
    };
    match var.get_modification_status() {
        ElviMutable::Normal => println!("declare -- {}={}", name, var.get_value()),
        ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
            println!("declare -r {}={}", name, var.get_value());
        }
    }
    ReturnCode::ret(ReturnCode::SUCCESS)
}
