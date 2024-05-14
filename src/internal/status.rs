use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub struct ReturnCode {
    val: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseReturnCodeError;

impl ReturnCode {
    pub fn ret(val: u16) -> Self {
        ReturnCode { val }
    }

    pub fn get(self) -> u16 {
        self.val
    }

    pub fn cap(self) -> u8 {
        (self.val % 256).try_into().unwrap()
    }

    pub const SUCCESS: u16 = 0;
    pub const FAILURE: u16 = 1;
    pub const MISUSE: u16 = 2;
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
