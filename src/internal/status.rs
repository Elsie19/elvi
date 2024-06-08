use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
/// Wrapper for a [`u16`] to represent an error code.
pub struct ReturnCode {
    /// Actual error code.
    val: u16,
}

#[derive(Debug, PartialEq, Eq)]
/// Struct for bad parses into a [`ReturnCode`].
pub struct ParseReturnCodeError;

/// <https://tldp.org/LDP/abs/html/exitcodes.html>
impl ReturnCode {
    /// Returns an instance of [`ReturnCode`] with a given value.
    pub fn ret(val: u16) -> Self {
        ReturnCode { val }
    }

    /// Pull [`ReturnCode::val`].
    pub fn get(self) -> u16 {
        self.val
    }

    /// Invert return code based on a boolean.
    ///
    /// `true` will invert.
    /// `false` will not invert.
    pub fn invert(self, invert: bool) -> Self {
        if invert {
            Self {
                val: if self.val == Self::SUCCESS {
                    Self::FAILURE
                } else {
                    Self::SUCCESS
                },
            }
        } else {
            Self {
                val: if self.val == Self::SUCCESS {
                    Self::SUCCESS
                } else {
                    Self::FAILURE
                },
            }
        }
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
    /// A script cannot be executed due to permission errors.
    pub const PERMISSION_DENIED: u16 = 126;
    /// A command not found in PATH that is called shall return 127.
    pub const COMMAND_NOT_FOUND: u16 = 127;
    /// When ctrl-C is called on a script.
    pub const CTRL_C: u16 = 130;
}

impl FromStr for ReturnCode {
    type Err = ParseReturnCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(x) = s.parse::<u16>() else {
            return Err(ParseReturnCodeError);
        };

        Ok(ReturnCode { val: x })
    }
}

impl From<bool> for ReturnCode {
    fn from(value: bool) -> Self {
        if value {
            Self { val: Self::SUCCESS }
        } else {
            Self { val: Self::FAILURE }
        }
    }
}

impl From<u16> for ReturnCode {
    fn from(value: u16) -> Self {
        Self { val: value }
    }
}

impl std::ops::Not for ReturnCode {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self.val {
            0 => Self { val: Self::FAILURE },
            _ => Self { val: Self::SUCCESS },
        }
    }
}
