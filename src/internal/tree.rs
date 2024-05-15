use super::variables::{ElviType, Variable};

#[derive(Debug)]
/// A list of possible actions a line can cause.
pub enum Actions {
    ChangeVariable((String, Variable)),
    Builtin(Builtins),
    Command(Vec<String>),
    Null,
}

#[derive(Debug)]
/// A list of builtins and their parameters.
pub enum Builtins {
    Dbg(String),
    Unset(String),
    Exit(Option<ElviType>),
}
