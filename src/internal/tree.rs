use super::variables::{ElviType, Variable};

#[derive(Debug)]
pub enum Actions {
    ChangeVariable((String, Variable)),
    Builtin(Builtins),
    Command(Vec<String>),
    Null,
}

#[derive(Debug)]
pub enum Builtins {
    Dbg(String),
    Exit(Option<ElviType>),
}
