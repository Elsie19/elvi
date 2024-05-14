use core::fmt;
use custom_error::custom_error;
use snailquote::{escape, unescape};
use std::collections::HashMap;

use super::status::ReturnCode;

custom_error! {pub VariableError
    Readonly{name:String, line:usize, column:usize} = "elvi: {name}: readonly variable (set on line '{line}' column '{line}')"
}

#[derive(Debug, Clone)]
pub enum ElviType {
    String(String),
    Number(usize),
    ErrExitCode(u8),
    // Array(Vec<Self>),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy)]
pub enum ElviMutable {
    Normal,
    Readonly,
    ReadonlyUnsettable,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElviGlobal {
    Global,
    Normal(u32),
}

#[derive(Debug, Clone)]
pub struct Variables {
    vars: HashMap<String, Variable>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    contents: ElviType,
    modification_status: ElviMutable,
    shell_lvl: ElviGlobal,
    line: (usize, usize),
}

impl Variables {
    pub fn default() -> Self {
        Self {
            vars: HashMap::from([
                (
                    "PS1".into(),
                    Variable {
                        contents: ElviType::String("$ ".into()),
                        modification_status: ElviMutable::Normal,
                        shell_lvl: ElviGlobal::Global,
                        line: (0, 0),
                    },
                ),
                (
                    "IFS".into(),
                    Variable {
                        contents: ElviType::String(r" \t\n".into()),
                        modification_status: ElviMutable::Normal,
                        shell_lvl: ElviGlobal::Global,
                        line: (0, 0),
                    },
                ),
                (
                    "PATH".into(),
                    Variable {
                        contents: ElviType::String("/usr/sbin:/usr/bin:/sbin:/bin".into()),
                        modification_status: ElviMutable::Normal,
                        shell_lvl: ElviGlobal::Global,
                        line: (0, 0),
                    },
                ),
                (
                    "?".into(),
                    Variable {
                        contents: ElviType::ErrExitCode(0),
                        modification_status: ElviMutable::ReadonlyUnsettable,
                        shell_lvl: ElviGlobal::Global,
                        line: (0, 0),
                    },
                ),
            ]),
        }
    }

    pub fn get_variable(&self, var: &str) -> Option<&Variable> {
        self.vars.get(var)
    }

    pub fn set_ret(&mut self, code: ReturnCode) {
        self.vars.insert(
            "?".into(),
            Variable::oneshot_var(
                ElviType::ErrExitCode(code.get()),
                ElviMutable::ReadonlyUnsettable,
                ElviGlobal::Global,
                (0, 0),
            ),
        );
    }

    pub fn set_variable(&mut self, name: String, var: Variable) -> Result<(), VariableError> {
        if let Some(value) = self.vars.get(&name) {
            let le_lines = value.clone();
            match value.modification_status {
                ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
                    self.set_ret(ReturnCode::ret(1));
                    Err(VariableError::Readonly {
                        name,
                        line: le_lines.line.0,
                        column: le_lines.line.1,
                    })
                }
                ElviMutable::Normal => {
                    self.set_ret(ReturnCode::ret(0));
                    self.vars.insert(name, var);
                    Ok(())
                }
            }
        // Is this a fresh variable?
        } else {
            self.set_ret(ReturnCode::ret(0));
            self.vars.insert(name, var);
            Ok(())
        }
    }
}

impl Variable {
    pub fn get_value(&self) -> &ElviType {
        &self.contents
    }

    pub fn oneshot_var(
        contents: ElviType,
        modification_status: ElviMutable,
        shell_lvl: ElviGlobal,
        line: (usize, usize),
    ) -> Self {
        Self {
            contents,
            modification_status,
            shell_lvl,
            line,
        }
    }

    pub fn get_lvl(&self) -> ElviGlobal {
        self.shell_lvl
    }

    pub fn change_lvl(&mut self, lvl: u32) -> ElviGlobal {
        self.shell_lvl = ElviGlobal::Normal(lvl);
        self.shell_lvl
    }

    pub fn get_modification_status(&self) -> ElviMutable {
        self.modification_status
    }
}

impl ElviType {
    pub fn eval_escapes(&self) -> ElviType {
        match self {
            ElviType::String(le_string) => ElviType::String(unescape(le_string).unwrap()),
            ElviType::Number(x) => ElviType::Number(*x),
            ElviType::Boolean(x) => ElviType::Boolean(*x),
            ElviType::ErrExitCode(x) => ElviType::ErrExitCode(*x),
        }
    }
}

impl fmt::Display for ElviType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElviType::String(x) => write!(f, "{}", escape(x)),
            ElviType::Number(x) => write!(f, "{x}"),
            ElviType::ErrExitCode(x) => write!(f, "{x}"),
            ElviType::Boolean(x) => write!(f, "{x}"),
        }
    }
}
