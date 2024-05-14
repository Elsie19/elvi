use crate::internal::status::ReturnCode;
use crate::internal::variables::Variable;
use crate::internal::variables::Variables;
use crate::internal::variables::{ElviGlobal, ElviMutable, ElviType};

pub fn builtin_dbg(name: String, variables: &mut Variables) -> ReturnCode {
    let var = match variables.get_variable(&name) {
        Some(x) => x,
        None => {
            eprintln!("dbg: no such variable: {}", name);
            return ReturnCode::ret(1);
        }
    };
    match var.get_modification_status() {
        ElviMutable::Normal => println!("declare -- {}={}", name, var.get_value()),
        ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
            println!("declare -r {}={}", name, var.get_value())
        }
    }
    ReturnCode::ret(0)
}
