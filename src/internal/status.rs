use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
/// Wrapper for a [`u16`] to represent an error code.
///
/// All documentation for exit codes are available at
/// <https://tldp.org/LDP/abs/html/exitcodes.html>.
pub struct ReturnCode {
    /// Actual error code.
    val: u16,
}

#[derive(Debug, PartialEq, Eq)]
/// Struct for bad parses into a [`ReturnCode`].
pub struct ParseReturnCodeError;

impl ReturnCode {
    /// Returns an instance of [`ReturnCode`] with a given value.
    #[must_use]
    pub fn ret(val: u16) -> Self {
        ReturnCode { val }
    }

    /// Pull [`ReturnCode::val`].
    #[must_use]
    pub fn get(self) -> u16 {
        self.val
    }

    /// Invert return code based on a boolean.
    ///
    /// `true` will invert.
    /// `false` will not invert.
    #[must_use]
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
    #[must_use]
    // We are ok with this, because by doing the `% 256`, we ensure it's in range.
    #[allow(clippy::cast_possible_truncation)]
    pub fn cap(self) -> u8 {
        (self.val % 256) as u8
    }

    /// Success
    ///
    /// # Code
    /// A successful command shall return `0`.
    pub const SUCCESS: u16 = 0;
    /// Failure
    ///
    /// # Code
    /// A general failure code shall return `1`.
    pub const FAILURE: u16 = 1;
    /// Misuse
    ///
    /// # Code
    /// Misuse of a builtin shall return `2`.
    pub const MISUSE: u16 = 2;
    /// Permission denied
    ///
    /// # Code
    /// A script cannot be executed due to permission errors shall return `126`.
    pub const PERMISSION_DENIED: u16 = 126;
    /// Command not found
    ///
    /// # Code
    /// A command not found in `PATH` that is called shall return `127`.
    pub const COMMAND_NOT_FOUND: u16 = 127;
    /// Signal killing
    ///
    /// # Code
    /// When ctrl-C is called on a script, it shall return `130`.
    pub const CTRL_C: u16 = 130;
}

impl FromStr for ReturnCode {
    type Err = ParseReturnCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(x) = s.parse::<u16>() {
            Ok(ReturnCode { val: x })
        } else {
            Err(ParseReturnCodeError)
        }
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

impl From<i32> for ReturnCode {
    fn from(value: i32) -> Self {
        // Reasonably sure this won't matter, I don't care about it losing bits from the top.
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        Self { val: value as u16 }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_out_of_bounds() {
        let out_bounds = 300;
        let errcode: ReturnCode = out_bounds.into();
        assert_eq!(errcode.cap(), 44);
    }

    #[test]
    fn not() {
        let errcode: ReturnCode = ReturnCode::FAILURE.into();
        assert_ne!(errcode, !errcode)
    }
}
