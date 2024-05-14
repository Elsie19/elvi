#[derive(Debug, Copy, Clone)]
pub struct ReturnCode {
    val: u8,
}

impl ReturnCode {
    pub fn ret(val: u8) -> Self {
        ReturnCode { val }
    }

    pub fn get(self) -> u8 {
        self.val
    }

    pub const SUCCESS: u8 = 0;
    pub const FAILURE: u8 = 1;
    pub const COMMAND_NOT_FOUND: u8 = 127;
    pub const CTRL_C: u8 = 130;
}
