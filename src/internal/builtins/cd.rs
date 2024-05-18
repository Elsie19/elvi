use crate::internal::status::ReturnCode;
use crate::internal::variables::{ElviType, Variables};

/// The internal code that runs when the `cd` builtin is run.
pub fn builtin_cd(flag: Option<ElviType>, variables: &mut Variables) -> ReturnCode {
    if flag.is_none() {
        match variables.set_variable(
            "OLDPWD".to_string(),
            variables.get_variable("PWD").unwrap().clone(),
        ) {
            Ok(()) => {}
            Err(oops) => eprintln!("{oops}"),
        }
        match variables.set_variable(
            "PWD".to_string(),
            variables.get_variable("HOME").unwrap().clone(),
        ) {
            Ok(()) => {}
            Err(oops) => eprintln!("{oops}"),
        }
        return ReturnCode::ret(ReturnCode::SUCCESS);
    }
    // Atp we know we got something, so let's check if it's `-` or a path.
    match flag.unwrap().to_string().as_str() {
        "-" => {
            let swap = variables.get_variable("PWD").unwrap().clone();
            match variables.set_variable(
                "PWD".to_string(),
                variables.get_variable("OLDPWD").unwrap().clone(),
            ) {
                Ok(()) => {}
                Err(oops) => eprintln!("{oops}"),
            }
            println!("{}", variables.get_variable("PWD").unwrap().get_value());
            match variables.set_variable("OLDPWD".to_string(), swap) {
                Ok(()) => {}
                Err(oops) => eprintln!("{oops}"),
            }
            ReturnCode::ret(ReturnCode::SUCCESS)
        }
        patho => {
            println!("Cding to {}", patho);
            ReturnCode::ret(ReturnCode::SUCCESS)
        }
    }
}
