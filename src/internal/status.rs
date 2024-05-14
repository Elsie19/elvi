#[derive(Debug)]
pub struct ReturnCode {
    val: u8,
}

impl ReturnCode {
    pub fn ret(val: u8) -> Self {
        ReturnCode { val }
    }

    pub fn get(&self) -> u8 {
        self.val
    }

    pub const SUCCESS: u8 = 0;
}
