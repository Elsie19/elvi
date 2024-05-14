use crate::internal::status::ReturnCode;
use crate::internal::variables::Variable;
use crate::internal::variables::Variables;
use crate::internal::variables::{ElviGlobal, ElviMutable, ElviType};

pub fn builtin_dbg(name: String, variables: &mut Variables) -> ReturnCode {
    let var = variables.get_variable(&name).unwrap();
    match var.get_modification_status() {
        ElviMutable::Normal => println!("declare -- {}={}", name, var.get_value()),
        ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
            println!("declare -r {}={}", name, var.get_value())
        }
    }
    ReturnCode::ret(0)
}
