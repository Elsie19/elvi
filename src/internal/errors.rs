use super::status::ReturnCode;

/// Supplies information relevant to errors.
pub trait ElviError {
    /// Give return code for a given error type.
    fn ret(&self) -> ReturnCode;
}

#[derive(Debug)]
/// Errors relating to commands.
pub enum CommandError {
    NotFound { name: String },
    SubCommandNotFound { name: &'static str, cmd: String },
    CannotCd { name: String, path: String },
    PermissionDenied { path: String },
}

impl std::error::Error for CommandError {}

impl ElviError for CommandError {
    fn ret(&self) -> ReturnCode {
        match self {
            Self::NotFound { .. } => ReturnCode::COMMAND_NOT_FOUND.into(),
            Self::PermissionDenied { .. } => ReturnCode::PERMISSION_DENIED.into(),
            Self::CannotCd { .. } => ReturnCode::MISUSE.into(),
            Self::SubCommandNotFound { .. } => ReturnCode::FAILURE.into(),
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NotFound { name } => write!(f, "elvi: {name}: not found"),
            Self::SubCommandNotFound { name, cmd } => write!(f, "elvi: {name}: {cmd}: not found"),
            Self::CannotCd { name, path } => write!(f, "elvi: {name}: can't cd to {path}"),
            Self::PermissionDenied { path } => write!(f, "elvi: {path}: Permission denied"),
        }
    }
}

#[derive(Debug)]
/// Errors relating to variables.
pub enum VariableError {
    Readonly { name: String, lines: (usize, usize) },
    IllegalNumber { name: String, caller: &'static str },
    NoSuchVariable { name: String, caller: &'static str },
}

impl std::error::Error for VariableError {}

impl ElviError for VariableError {
    fn ret(&self) -> ReturnCode {
        match self {
            Self::NoSuchVariable { .. } => ReturnCode::FAILURE.into(),
            Self::Readonly { .. } | Self::IllegalNumber { .. } => ReturnCode::MISUSE.into(),
        }
    }
}

impl std::fmt::Display for VariableError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NoSuchVariable { name, caller } => {
                write!(f, "{caller}: no such variable: {name}")
            }
            Self::Readonly { name, lines } => write!(
                f,
                "elvi: {name}: readonly variable (set on line '{}' column '{}')",
                lines.0, lines.1
            ),
            Self::IllegalNumber { name, caller } => {
                write!(f, "elvi: {caller}: Illegal number: {name})")
            }
        }
    }
}
