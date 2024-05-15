use crate::internal::{status::ReturnCode, variables::ElviType};

/// The internal code that runs when the `exit` builtin is run.
pub fn builtin_exit(num: Option<ElviType>) -> ReturnCode {
    match num {
        Some(yo) => {
            let try_code = yo.to_string().parse::<ReturnCode>();
            match try_code {
                Ok(yay) => return yay,
                Err(_) => {
                    eprintln!("elvi: exit: Illegal number: {yo}");
                    return ReturnCode::ret(ReturnCode::MISUSE);
                }
            }
        }
        None => return ReturnCode::ret(ReturnCode::SUCCESS),
    }
}
