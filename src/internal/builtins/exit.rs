use crate::internal::{status::ReturnCode, variables::ElviType};

/// The internal code that runs when the `exit` builtin is run.
///
/// This builtin corresponds with
/// <https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/utilities/V3_chap02.html#exit>
/// except for trapping.
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
        // If we have a bare exit, return with 0.
        None => return ReturnCode::ret(ReturnCode::SUCCESS),
    }
}
