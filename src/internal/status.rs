use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub struct ReturnCode {
    val: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseReturnCodeError;

/// <https://tldp.org/LDP/abs/html/exitcodes.html>
impl ReturnCode {
    pub fn ret(val: u16) -> Self {
        ReturnCode { val }
    }

    pub fn get(self) -> u16 {
        self.val
    }

    /// <https://tldp.org/LDP/abs/html/exitcodes.html#AEN23629>
    /// Will cap a given return value to within a range of 0-255.
    pub fn cap(self) -> u8 {
        (self.val % 256).try_into().unwrap()
    }

    /// Success in POSIX is defined as 0.
    pub const SUCCESS: u16 = 0;
    /// Failure in POSIX is defined as 1.
    pub const FAILURE: u16 = 1;
    /// Misuse of a builtin shall return 2.
    pub const MISUSE: u16 = 2;
    /// A command not found in PATH that is called shall return 127.
    pub const COMMAND_NOT_FOUND: u16 = 127;
    pub const CTRL_C: u16 = 130;
}

impl FromStr for ReturnCode {
    type Err = ParseReturnCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(x) = s.parse::<u16>() else { return Err(ParseReturnCodeError) };

        Ok(ReturnCode { val: x })
    }
}
