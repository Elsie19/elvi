use core::fmt;
use custom_error::custom_error;
use pest_consume::Itertools;
use snailquote::{escape, unescape};
use std::{collections::HashMap, env};

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
    /// A type that signifies command substitution, cannot be assigned inside of Elvi.
    CommandSubstitution(String),
    /// A type that signifies the need to evaluate variables. This will have to be converted to
    /// [`ElviType::String`] when it is seen. By nature, it has to be a double quoted string.
    VariableSubstitution(String),
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
    pub fn get_variable(&self, var: &str) -> Option<&Variable> {
        self.vars.get(var)
    }

    /// Return a hashmap of all variables marked as [`ElviGlobal::Global`] and their corresponding
    /// values (can only be a [`ElviType::String`]).
    pub fn get_environmentals(&self) -> HashMap<String, String> {
        let mut ret: HashMap<String, String> = HashMap::new();
        for (name, var) in self.vars.iter() {
            if var.get_lvl() == ElviGlobal::Global {
                ret.insert(name.to_string(), var.get_value().to_string());
            }
        }
        ret
    }

    /// Unsets a variable.
    ///
    /// # Notes
    /// Check before running this function if a varible is not [`ElviMutable::Normal`], because
    /// this function will happily unset anything.
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
    ///
    /// # Errors
    /// Will return [`VariableError`] if a variable is [`ElviMutable::Readonly`] or
    /// [`ElviMutable::ReadonlyUnsettable`].
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

impl Default for Variables {
    /// Create new default variable list with required variables.
    fn default() -> Self {
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
                (
                    "PWD".into(),
                    Variable {
                        contents: ElviType::String(
                            env::current_dir().unwrap().to_str().unwrap().to_string(),
                        ),
                        modification_status: ElviMutable::Normal,
                        shell_lvl: ElviGlobal::Global,
                        line: (0, 0),
                    },
                ),
            ]),
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

    /// Return the [`ElviGlobal`] of a given variable.
    pub fn get_lvl(&self) -> ElviGlobal {
        self.shell_lvl
    }

    /// Change any variable's level to a [`ElviGlobal::Normal`] and return it.
    pub fn change_lvl(&mut self, lvl: u32) -> ElviGlobal {
        self.shell_lvl = ElviGlobal::Normal(lvl);
        self.shell_lvl
    }

    /// Get the [`ElviMutable`] of a variable.
    pub fn get_modification_status(&self) -> ElviMutable {
        self.modification_status
    }

    /// Get the assignment line of a variable.
    pub fn get_line(&self) -> (usize, usize) {
        self.line
    }

    /// Change and return the contents of a variable.
    pub fn change_contents(&mut self, var: ElviType) -> &Variable {
        self.contents = var;
        self
    }
}

impl ElviType {
    /// Return an escaped string using [`snailquote::unescape`].
    pub fn eval_escapes(&self) -> Self {
        match self {
            Self::String(le_string) => Self::String(unescape(le_string).unwrap()),
            Self::VariableSubstitution(le_string) => {
                Self::VariableSubstitution(unescape(le_string).unwrap())
            }
            default => default.clone(),
        }
    }

    /// This assumes [`ElviType::VariableSubstitution`].
    pub fn eval_variables(&self, vars: &Variables) -> Self {
        match self {
            // So basically because of my shitty thinking, we set all double quotes to
            // [`ElviType::VariableSubstitution`] and convert that into a string. Haha.
            ElviType::VariableSubstitution(le_string) => {
                // Let's skip the variables loops if we can't even find anything.
                if !le_string.contains('$') && !le_string.contains(r"\$") {
                    return Self::String(le_string.to_string());
                }
                let mut chars_of = le_string.chars().peekable();
                let mut back_to_string = String::new();
                while let Some(charp) = chars_of.next() {
                    // Do we have a normal string please.
                    if charp != '$' {
                        back_to_string.push(charp);
                        continue;
                    // Do we have a variable that is escaped?
                    } else if charp == '\\' && chars_of.peek() == Some(&'$') {
                        back_to_string.push(charp);
                        chars_of.next().unwrap();
                        back_to_string.push(charp);
                        continue;
                    }
                    // Ok at this point we have a variable! Woo, yay, congrats. Now is it a stupid
                    // mfing $bare_variable or a lovely (we love) ${braced_variable}?
                    // Oh and we're at '$' in the thing.
                    if chars_of.peek() == Some(&'{') {
                        // WOOOOOOOO
                        chars_of.next().unwrap();
                        //BUG:  What about ${foo\}bar}
                        //TODO: Work on parameter expansion, ugh.
                        let tasty_var: String =
                            chars_of.by_ref().take_while(|&c| c != '}').collect();
                        match vars.get_variable(&tasty_var) {
                            Some(woot) => {
                                // This is the magic of the entire function lol. Bask in its
                                // glory!!
                                back_to_string.push_str(woot.get_value().to_string().as_str())
                            }
                            None => {}
                        }
                        continue;
                    } else {
                        // Fuck.
                        // Well before we fuck we should check if this is the last character, which
                        // is stupid but hey, I'm writing a POSIX shell.
                        if chars_of.peek() == None {
                            back_to_string.push('$');
                            continue;
                        }
                        // Ok now we fuck
                        let tasty_var: String = chars_of
                            .by_ref()
                            // We don't wanna consume the character it fails on, otherwise we'd use
                            // take_while() instead.
                            .peeking_take_while(|&c| c != ' ' && c != '-' && c != '\\')
                            .collect();
                        match vars.get_variable(&tasty_var) {
                            Some(woot) => {
                                back_to_string.push_str(woot.get_value().to_string().as_str());
                            }
                            None => {}
                        }
                    }
                }
                Self::String(back_to_string)
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
            ElviType::CommandSubstitution(x) => write!(f, "{x}"),
            ElviType::VariableSubstitution(x) => write!(f, "{x}"),
        }
    }
}
