use core::fmt;
use custom_error::custom_error;
use snailquote::{escape, unescape};
use std::collections::HashMap;

use super::status::ReturnCode;

custom_error! {pub VariableError
    Readonly{name:String, line:usize, column:usize} = "elvi: {name}: readonly variable (set on line '{line}' column '{line}')",
    IllegalNumber{name:String, caller:String} = "elvi: {caller}: Illegal number: {name})",
    NoSuchVariable{name:String, caller:String} = "{caller}: no such variable: {name})",
}

#[derive(Debug, Clone)]
/// Struct representing the variable types in Elvi.
pub enum ElviType {
    /// A string.
    String(String),
    /// A number.
    Number(usize),
    /// An error code type, cannot be assigned inside of Elvi.
    ErrExitCode(u16),
    /// Boolean.
    Boolean(bool),
}

#[derive(Debug, Clone, Copy)]
/// Enum representing the state that a variable can be.
pub enum ElviMutable {
    /// Mutable variable, the default one.
    Normal,
    /// Readonly variable, cannot be unset or changed.
    Readonly,
    /// Generally only used internally, will create a readonly variable that can be unset.
    ReadonlyUnsettable,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Enum representing the globality of a variable.
pub enum ElviGlobal {
    /// Exported variable.
    Global,
    /// Scoped variable.
    Normal(u32),
}

#[derive(Debug, Clone)]
/// Global variable list.
pub struct Variables {
    /// Hashmap of a variable name and its contents.
    vars: HashMap<String, Variable>,
}

#[derive(Debug, Clone)]
/// Single variable content.
pub struct Variable {
    /// Contents of the variable.
    contents: ElviType,
    /// Mutability status.
    modification_status: ElviMutable,
    /// Relation to `$SHLVL`.
    shell_lvl: ElviGlobal,
    /// Line and column variable was declared on.
    line: (usize, usize),
}

impl Variables {
    /// Create new default variable list with required variables.
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

    pub fn unset(&mut self, var: &str) -> Option<()> {
        match self.vars.remove(var) {
            Some(_) => Some(()),
            None => None,
        }
    }

    /// Quick function to set `$?`.
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

    /// Set a given variable.
    pub fn set_variable(&mut self, name: String, var: Variable) -> Result<(), VariableError> {
        if let Some(value) = self.vars.get(&name) {
            let le_lines = value.clone();
            match value.modification_status {
                ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
                    self.set_ret(ReturnCode::ret(ReturnCode::FAILURE));
                    Err(VariableError::Readonly {
                        name,
                        line: le_lines.line.0,
                        column: le_lines.line.1,
                    })
                }
                ElviMutable::Normal => {
                    self.set_ret(ReturnCode::ret(ReturnCode::SUCCESS));
                    self.vars.insert(name, var);
                    Ok(())
                }
            }
        // Is this a fresh variable?
        } else {
            self.set_ret(ReturnCode::ret(ReturnCode::SUCCESS));
            self.vars.insert(name, var);
            Ok(())
        }
    }
}

impl Variable {
    pub fn get_value(&self) -> &ElviType {
        &self.contents
    }

    /// Return a variable template with all options available.
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

    pub fn get_line(&self) -> (usize, usize) {
        self.line
    }
}

impl ElviType {
    /// Return an escaped string using [`snailquote::unescape`].
    pub fn eval_escapes(&self) -> ElviType {
        match self {
            ElviType::String(le_string) => ElviType::String(unescape(le_string).unwrap()),
            default => default.clone(),
        }
    }

    // This assumes double quotes.
    pub fn eval_variables(&self, vars: &Variables) -> ElviType {
        match self {
            ElviType::String(le_string) => {
                let chars_of = le_string.chars();
                unimplemented!();
            }
            default => default.clone(),
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
