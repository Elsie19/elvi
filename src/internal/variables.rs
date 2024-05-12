use custom_error::custom_error;
use std::collections::HashMap;

custom_error! {pub VariableError
    Readonly{name:String} = "Cannot modify readonly variable: '{name}'"
}

#[derive(Debug)]
pub enum ElviType {
    String(String),
    Array(Vec<Self>),
    Boolean(bool),
}

#[derive(Debug)]
pub enum ElviMutable {
    Normal,
    Readonly,
    ReadonlyUnsettable,
}

#[derive(Debug)]
pub enum ElviGlobal {
    Global,
    Normal(u32),
}

#[derive(Debug)]
pub struct Variables {
    vars: HashMap<String, Variable>,
}

#[derive(Debug)]
pub struct Variable {
    contents: ElviType,
    modification_status: ElviMutable,
    shell_lvl: ElviGlobal,
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
                    },
                ),
                (
                    "IFS".into(),
                    Variable {
                        contents: ElviType::String(r#" \t\n"#.into()),
                        modification_status: ElviMutable::Normal,
                        shell_lvl: ElviGlobal::Global,
                    },
                ),
                (
                    "PATH".into(),
                    Variable {
                        contents: ElviType::String("/usr/sbin:/usr/bin:/sbin:/bin".into()),
                        modification_status: ElviMutable::Normal,
                        shell_lvl: ElviGlobal::Global,
                    },
                ),
            ]),
        }
    }

    pub fn get_variable(&self, var: String) -> Option<&Variable> {
        self.vars.get(&var)
    }

    pub fn set_variable(&mut self, name: String, var: Variable) -> Result<(), VariableError> {
        match self.vars.get(&name) {
            Some(value) => match value.modification_status {
                ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
                    Err(VariableError::Readonly { name })
                }
                ElviMutable::Normal => {
                    self.vars.insert(name, var).unwrap();
                    return Ok(());
                }
            },
            None => {
                self.vars.insert(name, var).unwrap();
                return Ok(());
            }
        }
    }
}

impl Variable {
    pub fn get_value(&self) -> &ElviType {
        &self.contents
    }
}
