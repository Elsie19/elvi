use crate::internal::status::ReturnCode;

pub fn builtin_exit(num: String) -> ReturnCode {
    let try_code = num.parse::<ReturnCode>();
    match try_code {
        Ok(yay) => return yay,
        Err(_) => {
            eprintln!("elvi: exit: Illegal number: {num}");
            return ReturnCode::ret(ReturnCode::MISUSE);
        }
    }
}
