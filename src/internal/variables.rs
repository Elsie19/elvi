use custom_error::custom_error;
use snailquote::unescape;
use std::collections::HashMap;

custom_error! {pub VariableError
    Readonly{name:String, line:usize, column:usize} = "elvi: {name}: readonly variable (set on line '{line}' column '{line}')"
}

#[derive(Debug, Clone)]
pub enum ElviType {
    String(String),
    // Array(Vec<Self>),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy)]
pub enum ElviMutable {
    Normal,
    Readonly,
    ReadonlyUnsettable,
}

#[derive(Debug, Clone, Copy)]
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
                        contents: ElviType::String(r#" \t\n"#.into()),
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
            ]),
        }
    }

    pub fn get_variable(&self, var: &str) -> Option<&Variable> {
        self.vars.get(var)
    }

    pub fn set_variable(&mut self, name: String, var: Variable) -> Result<(), VariableError> {
        if let Some(value) = self.vars.get(&name) {
            match value.modification_status {
                ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
                    Err(VariableError::Readonly {
                        name,
                        line: value.line.0,
                        column: value.line.1,
                    })
                }
                ElviMutable::Normal => {
                    self.vars.insert(name, var);
                    Ok(())
                }
            }
        // Is this a fresh variable?
        } else {
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
}

impl ElviType {
    pub fn eval_escapes(&self) -> Option<ElviType> {
        match self {
            ElviType::String(le_string) => Some(ElviType::String(unescape(le_string).unwrap())),
            _ => None,
        }
    }
}
