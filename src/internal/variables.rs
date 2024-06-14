use core::fmt;
use glob::glob;
use homedir::get_home;
use pest_consume::Itertools;
use regex::Regex;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process,
};

use super::errors::VariableError;
use super::status::ReturnCode;

#[derive(Debug, Clone, PartialEq)]
/// Struct representing the variable types in Elvi.
pub enum ElviType {
    /// A string (a single quoted string or post variable/command substituted string).
    ///
    /// If this string is seen, take it as it is ;)
    String(String),
    /// A number.
    Number(usize),
    /// An error code type, **cannot be assigned inside of an Elvi script**.
    ErrExitCode(u16),
    /// Boolean.
    Boolean(bool),
    /// A type that signifies command substitution, **cannot be assigned inside of  an Elvi script**.
    CommandSubstitution(String),
    /// A type that signifies the need to evaluate variables. This will have to be converted to
    /// [`ElviType::String`] when it is seen. By nature, it has to be a double quoted string.
    VariableSubstitution(String),
    /// A string that may or may not be needed to expand.
    ///
    /// ```bash
    /// foo
    /// ${bang}
    /// $borp
    /// ```
    BareString(String),
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
    ///
    /// Every variable in [`Variables`] that is above the current level will be sweeped after the
    /// subshell ends.
    Normal(u32),
}

#[derive(Debug, Clone)]
/// Global variable list.
pub struct Variables {
    /// Hashmap of a variable name and its contents.
    vars: HashMap<String, Variable>,
    /// A separate field used solely for positional parameters.
    params: Vec<Variable>,
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
    #[must_use]
    /// Get a variable.
    ///
    /// # Notes
    /// This method has a bit of indirection to it. If it detects what appears to be a positional
    /// parameter, it will silently pull that, even though it is not coming from the vars list.
    pub fn get_variable(&self, var: &str) -> Option<&Variable> {
        if let Ok(pos) = var.parse::<usize>() {
            self.params.get(pos)
        } else {
            self.vars.get(var)
        }
    }

    /// Return a hashmap of all variables marked as [`ElviGlobal::Global`] and their corresponding
    /// values (can only be a [`ElviType::String`]).
    #[must_use]
    pub fn get_environmentals(&self) -> HashMap<String, String> {
        let mut ret: HashMap<String, String> = HashMap::new();
        for (name, var) in &self.vars {
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
    ///
    /// # Returns
    /// `Some(())` for a variable that was found and removed.
    ///
    /// `None` for a variable that was not found.
    pub fn unset(&mut self, var: &str) -> Option<()> {
        match self.vars.remove(var) {
            Some(_) => Some(()),
            None => None,
        }
    }

    /// Quick function to set `$?`.
    ///
    /// # Examples
    /// ```
    /// use super::internal::variables::Variables;
    /// use super::status::ReturnCode;
    /// let mut variables = Variables::default();
    /// let ret = 1;
    /// variables.set_ret(ReturnCode::ret(ret));
    /// ```
    pub fn set_ret(&mut self, code: ReturnCode) {
        self.vars.insert(
            "?".into(),
            Variable::oneshot_var(
                &ElviType::ErrExitCode(code.get()),
                ElviMutable::ReadonlyUnsettable,
                ElviGlobal::Global,
                (0, 0),
            ),
        );
    }

    /// Quick function to pull `$?`.
    ///
    /// # Panics
    /// Will only fail in a catastrophic situation where `$?` doesn't exist.
    #[must_use]
    pub fn get_ret(&self) -> ElviType {
        // This cannot fail on unwrap unless something awful happened.
        self.vars
            .get("?")
            .expect("Something very bad happened where `$?` does not exist")
            .get_value()
            .to_owned()
    }

    /// Set a given variable.
    ///
    /// # Errors
    /// Will return [`VariableError`] if a variable is [`ElviMutable::Readonly`] or
    /// [`ElviMutable::ReadonlyUnsettable`].
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

    /// Create a new set of parameters.
    pub fn new_parameters(&mut self, params: &[Variable]) {
        self.params = params.to_vec();
    }

    /// Pull parameters out.
    pub fn pull_parameters(&self) -> Vec<Variable> {
        self.params.clone()
    }

    /// Get count of parameters.
    pub fn len_parameters(&self) -> usize {
        self.params.len()
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
                (
                    "OLDPWD".into(),
                    Variable {
                        contents: ElviType::String(
                            env::current_dir().unwrap().to_str().unwrap().to_string(),
                        ),
                        modification_status: ElviMutable::Normal,
                        shell_lvl: ElviGlobal::Global,
                        line: (0, 0),
                    },
                ),
                (
                    "HOME".into(),
                    Variable {
                        contents: ElviType::String(
                            // We (I) don't support windows.
                            #[allow(deprecated)]
                            env::home_dir().unwrap().to_str().unwrap().to_string(),
                        ),
                        modification_status: ElviMutable::Normal,
                        shell_lvl: ElviGlobal::Global,
                        line: (0, 0),
                    },
                ),
            ]),

            params: vec![],
        }
    }
}

impl Default for Variable {
    fn default() -> Self {
        Self {
            contents: ElviType::String("".into()),
            modification_status: ElviMutable::Normal,
            shell_lvl: ElviGlobal::Global,
            line: (0, 0),
        }
    }
}

impl From<String> for Variable {
    fn from(value: String) -> Self {
        Self {
            contents: ElviType::String(value),
            ..Default::default()
        }
    }
}

impl Variable {
    #[must_use]
    /// Get [`ElviType`] of variable.
    pub fn get_value(&self) -> &ElviType {
        &self.contents
    }

    /// Return a variable template with all options available.
    #[must_use]
    pub fn oneshot_var(
        contents: &ElviType,
        modification_status: ElviMutable,
        shell_lvl: ElviGlobal,
        line: (usize, usize),
    ) -> Self {
        Self {
            contents: contents.clone(),
            modification_status,
            shell_lvl,
            line,
        }
    }

    /// Return the [`ElviGlobal`] of a given variable.
    #[must_use]
    pub fn get_lvl(&self) -> ElviGlobal {
        self.shell_lvl
    }

    /// Change any variable's level to a [`ElviGlobal::Normal`] and return it.
    pub fn change_lvl(&mut self, lvl: u32) -> ElviGlobal {
        self.shell_lvl = ElviGlobal::Normal(lvl);
        self.shell_lvl
    }

    /// Get the [`ElviMutable`] of a variable.
    #[must_use]
    pub fn get_modification_status(&self) -> ElviMutable {
        self.modification_status
    }

    /// Get the assignment line of a variable.
    #[must_use]
    pub fn get_line(&self) -> (usize, usize) {
        self.line
    }

    /// Change and return the contents of a variable.
    pub fn change_contents(&mut self, var: ElviType) -> &Self {
        self.contents = var;
        self
    }
}

impl ElviType {
    /// Return an escaped string using [`backslash::escape_ascii`].
    #[must_use]
    pub fn eval_escapes(&self) -> Self {
        match self {
            Self::VariableSubstitution(le_string) => {
                Self::VariableSubstitution(backslash::escape_ascii(le_string).unwrap())
            }
            default => default.clone(),
        }
    }

    /// Convert a [`ElviType::ErrExitCode`] into a [`ReturnCode`].
    #[must_use]
    pub fn convert_err_type(&self) -> ReturnCode {
        match self {
            Self::ErrExitCode(val) => (*val).into(),
            _ => unreachable!("What sorta idiot did that..."),
        }
    }

    /// Tilde substitution.
    ///
    /// # What it does
    /// 1. Converts `~` -> `$HOME`
    /// 2. Converts `~/foo` -> `$HOME/foo`
    /// 3. Converts `~bob/foo` -> `/home/bob/foo`
    /// 4. Converts `~bob` -> `/home/bob`
    ///
    /// # Panics
    /// This can panic if a username is found but a path cannot be found for it. You will probably
    /// *never* encounter this, and if you do, you have bigger issues than a panic.
    ///
    /// # Notes
    /// Requires [`ElviType::String`].
    #[must_use]
    pub fn tilde_expansion(&self, vars: &Variables) -> Self {
        match self {
            Self::String(le_string) => {
                let re = Regex::new(r"^~([a-z_][a-z0-9_]{0,30})").unwrap();
                let path = PathBuf::from(le_string);
                // So in POSIX, you can have two (*three) forms:
                //
                // ```bash
                // ~/foo
                // ~henry/oof
                // ~
                // ```
                // Do we have a tilde at the start?
                if path.starts_with("~/") {
                    let home_dir = vars.get_variable("HOME").unwrap().get_value();
                    let final_cd = home_dir.to_string()
                        + std::path::MAIN_SEPARATOR_STR
                        + path.strip_prefix("~/").unwrap().to_str().unwrap();
                    match self {
                        Self::String(_) => Self::String(final_cd),
                        Self::VariableSubstitution(_) => Self::VariableSubstitution(final_cd),
                        _ => unreachable!("Not possible."),
                    }
                // Perchance could it be a user form?
                } else if re.is_match(match path.parent() {
                    Some(p) => p.to_str().unwrap(),
                    None => "/".into(),
                }) {
                    let user = match path.parent() {
                        Some(woot) => {
                            if woot == Path::new("") {
                                path.to_str().unwrap()[1..].to_string()
                            } else {
                                // ~foo/bar -> foo
                                path.parent().unwrap().to_str().unwrap()[1..].to_string()
                            }
                        }
                        None => path.to_str().unwrap().to_string(),
                    };
                    let user_path = match get_home(&user) {
                        Ok(woot) => match woot {
                            Some(yas) => yas,
                            None => match self {
                                // We should return the literal path they provided
                                Self::String(_) => {
                                    return Self::String(path.to_str().unwrap().to_string());
                                }
                                Self::VariableSubstitution(_) => {
                                    return Self::VariableSubstitution(
                                        path.to_str().unwrap().to_string(),
                                    );
                                }
                                _ => unreachable!("Not possible."),
                            },
                        },
                        Err(oof) => panic!("Could not obtain this home directory LMAO {oof}"),
                    };
                    let final_cd = user_path.join(path.strip_prefix(format!("~{user}")).unwrap());
                    match self {
                        Self::String(_) => Self::String(final_cd.to_str().unwrap().to_string()),
                        Self::VariableSubstitution(_) => {
                            Self::VariableSubstitution(final_cd.to_str().unwrap().to_string())
                        }
                        _ => unreachable!("Not possible."),
                    }
                } else {
                    // We don't and the caller is an idiot. Congrats: here's your string back to
                    // you. Fuck you.
                    match self {
                        goopy @ (Self::String(_) | Self::VariableSubstitution(_)) => goopy.clone(),
                        _ => unreachable!(
                            "We already matched above. How did self change? It's immutable????"
                        ),
                    }
                }
            }
            default => default.clone(),
        }
    }

    /// This assumes [`ElviType::VariableSubstitution`]. If not, it will return the text given as
    /// is.
    #[must_use]
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
                    // Do we have a variable that is escaped?
                    if charp == '\\' && chars_of.peek() == Some(&'$') {
                        chars_of.next().unwrap();
                        back_to_string.push('$');
                    // Do we have a normal string please.
                    } else if charp != '$' {
                        back_to_string.push(charp);
                    } else {
                        // Ok at this point we have a variable! Woo, yay, congrats. Now is it a stupid
                        // mfing $bare_variable or a lovely (we love) ${braced_variable}?
                        // Oh and we're at '$' in the thing.
                        if chars_of.peek() == Some(&'{') {
                            // WOOOOOOOO
                            chars_of.next().unwrap();
                            //BUG:  What about ${foo\}bar}
                            let tasty_var: String =
                                chars_of.by_ref().take_while(|&c| c != '}').collect();
                            // So here we have a list of expanded special vars, but assuming that
                            // there are no special vars, we got len() == 0.
                            let expanded_out = self.eval_special_variable(&tasty_var, vars);
                            if !expanded_out.is_empty() {
                                for part in expanded_out {
                                    back_to_string.push_str(part.as_str());
                                }
                            }
                        } else {
                            // Fuck.
                            // Well before we fuck we should check if this is the last character, which
                            // is stupid but hey, I'm writing a POSIX shell.
                            if chars_of.peek().is_none() {
                                back_to_string.push('$');
                                continue;
                            }
                            // Ok now we fuck
                            let tasty_var: String = chars_of
                                .by_ref()
                                // We don't wanna consume the character it fails on, otherwise we'd use
                                // take_while() instead.
                                .peeking_take_while(|&c| {
                                    // TODO: Figure out how to work around `-` for it's special
                                    // parameter.
                                    // TODO: Also I need to figure out a better system than this.
                                    c != ' ' && c != '\\' && c != ':' && c != '-'
                                })
                                .collect();
                            let expanded_out = self.eval_special_variable(&tasty_var, vars);
                            if !expanded_out.is_empty() {
                                for part in expanded_out {
                                    back_to_string.push_str(part.as_str());
                                }
                            }
                        }
                    }
                }
                Self::String(back_to_string).eval_escapes()
            }
            default => default.clone(),
        }
    }

    /// This is for
    /// <https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#2.5.2>
    /// We also do variable expansion regardless in here.
    #[must_use]
    pub fn eval_special_variable(&self, var: &str, variables: &Variables) -> Vec<String> {
        let mut ret_vec = vec![];
        match var {
            // TODO: Fully flesh out the differences between these two.
            "@" | "*" => {
                let mut pushback = vec![];
                let loopo = variables.pull_parameters();
                for (idx, part) in loopo.iter().enumerate().skip(1) {
                    pushback.push(part.get_value().to_string());
                    if idx != loopo.len() {
                        pushback.push(" ".to_string());
                    }
                }
                return pushback;
            }
            "$" => {
                ret_vec.push(process::id().to_string());
            }
            "#" => ret_vec.push((variables.len_parameters() - 1).to_string()),
            default => {
                if let Some(woot) = variables.get_variable(default) {
                    ret_vec.push(woot.get_value().to_string());
                }
            }
        }
        ret_vec
    }

    /// Expand globs using [`glob()`].
    ///
    /// # Notes
    /// This function will return a list of paths it matches against, but if it does not match
    /// anything, it will return the literal string that it received.
    #[must_use = "Why no using the globs ya goof"]
    pub fn expand_globs(&self) -> Vec<Self> {
        let mut ret_vec = vec![];
        match glob(&self.to_string()) {
            Ok(paths) => {
                for patho in paths {
                    match patho {
                        Ok(yay) => {
                            ret_vec.push(ElviType::String(yay.to_str().unwrap().to_string()));
                        }
                        Err(boo) => eprintln!("{boo}"),
                    }
                }
                // So because of the [`glob`] library, any path that doesn't match a filesystem
                // path will be silently skipped, so what we're doing here is checking if that
                // event happened, and pushing the literal string back, to conform to:
                // ```shell
                // for i in /foobar/*; do
                //   echo "${i}"
                // done
                // # Prints: /foobar/*
                // ```
                if ret_vec.is_empty() {
                    ret_vec.push(ElviType::String(self.to_string()));
                }
            }
            Err(oof) => eprintln!("{oof}"),
        }
        ret_vec
    }
}

impl fmt::Display for ElviType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElviType::String(x)
            | ElviType::VariableSubstitution(x)
            | ElviType::CommandSubstitution(x)
            | ElviType::BareString(x) => write!(f, "{x}"),
            ElviType::Number(x) => write!(f, "{x}"),
            ElviType::ErrExitCode(x) => write!(f, "{x}"),
            ElviType::Boolean(x) => write!(f, "{x}"),
        }
    }
}

#[derive(Debug, Clone)]
/// Struct for global arguments
pub struct Arguments {
    pub args: Vec<String>,
}

impl From<Vec<String>> for Arguments {
    fn from(value: Vec<String>) -> Self {
        Self { args: value }
    }
}
