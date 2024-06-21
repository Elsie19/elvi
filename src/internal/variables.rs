use core::fmt;
use glob::glob;
use homedir::get_home;
use pest_consume::Itertools;
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    process,
};

use super::errors::VariableError;
use super::status::ReturnCode;

/// Functions to describe the quoted nature of a type.
pub trait QuotedNature {
    /// Is a type quoted or not.
    fn is_quoted(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
/// Struct representing the variable types in Elvi.
pub enum ElviType {
    /// A thin interface for [`String`]. This type should be taken exactly as a String
    /// would be taken.
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

impl QuotedNature for ElviType {
    fn is_quoted(&self) -> bool {
        !matches!(self, Self::String(_) | Self::BareString(_))
    }
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
    /// Local variable (**Not POSIX**).
    Local,
}

#[derive(Debug, Clone)]
/// Global variable list.
pub struct Variables {
    /// Hashmap of a variable name and its contents.
    pub vars: HashMap<String, Variable>,
    /// A separate field used solely for positional parameters.
    pub params: Vec<Variable>,
}

#[derive(Debug, Clone)]
/// Single variable content.
pub struct Variable {
    /// Contents of the variable.
    pub contents: ElviType,
    /// Mutability status.
    pub modification_status: ElviMutable,
    /// Relation to `$SHLVL`.
    pub shell_lvl: ElviGlobal,
    /// Line and column variable was declared on.
    pub line: (usize, usize),
}

impl Variables {
    #[must_use]
    /// Get a variable.
    ///
    /// # Notes
    /// This method has a bit of indirection to it. If it detects what appears to be a positional
    /// parameter, it will silently pull that, even though it is not coming from the vars list.
    pub fn get_variable(&self, var: impl Into<String>) -> Option<&Variable> {
        let var_two = var.into();
        if let Ok(pos) = var_two.parse::<usize>() {
            self.params.get(pos)
        } else {
            self.vars.get(&var_two)
        }
    }

    /// Return a hashmap of all variables marked as [`ElviGlobal::Global`] and their corresponding
    /// values (can only be a [`ElviType::String`]).
    #[must_use]
    pub fn get_environmentals(&self) -> HashMap<String, String> {
        self.vars
            .iter()
            .filter(|(_, var)| var.shell_lvl == ElviGlobal::Global)
            .map(|(name, var)| (name.to_string(), var.contents.to_string()))
            .collect()
    }

    /// Unsets a variable.
    ///
    /// # Notes
    /// Check before running this function if a variable is not [`ElviMutable::Normal`], because
    /// this function will happily unset anything.
    ///
    /// # Returns
    /// 1. `Some(())` for a variable that was found and removed.
    /// 2. `None` for a variable that was not found.
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
            Variable {
                contents: ElviType::ErrExitCode(code.get()),
                modification_status: ElviMutable::ReadonlyUnsettable,
                ..Default::default()
            },
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
            .contents
            .clone()
    }

    /// Set a given variable.
    ///
    /// # Errors
    /// Will return [`VariableError`] if a variable is [`ElviMutable::Readonly`] or
    /// [`ElviMutable::ReadonlyUnsettable`].
    pub fn set_variable(
        &mut self,
        name: impl Into<String>,
        var: impl Into<Variable>,
    ) -> Result<(), VariableError> {
        let name_two = name.into();
        let var_two = var.into();
        if let Some(value) = self.vars.get(&name_two) {
            match value.modification_status {
                ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
                    Err(VariableError::Readonly {
                        name: name_two,
                        lines: value.line,
                    })
                }
                ElviMutable::Normal => {
                    self.set_ret(ReturnCode::ret(ReturnCode::SUCCESS));
                    self.vars.insert(name_two, var_two);
                    Ok(())
                }
            }
        // Is this a fresh variable?
        } else {
            self.set_ret(ReturnCode::ret(ReturnCode::SUCCESS));
            self.vars.insert(name_two, var_two);
            Ok(())
        }
    }

    /// Create a new set of parameters.
    ///
    /// # Examples
    /// ```rust
    /// # use std::env;
    /// let args: Vec<String> = env::args().map(|var| var.to_owned().into()).collect();
    /// let mut variables = Variables::default();
    /// variables.new_parameters(params.as_slice());
    /// ```
    pub fn new_parameters(&mut self, params: &[Variable]) {
        self.params = params.to_vec();
    }

    /// Pull parameters out.
    #[must_use]
    pub fn pull_parameters(&self) -> Vec<Variable> {
        self.params.clone()
    }

    /// Get count of parameters.
    #[must_use]
    pub fn len_parameters(&self) -> usize {
        self.params.len()
    }
}

impl Default for Variables {
    /// Create new default variable list with required variables:
    ///
    /// * `PS1`
    /// * `IFS`
    /// * `PATH`
    /// * `?`
    /// * `PWD`
    /// * `OLDPWD`
    /// * `HOME`
    fn default() -> Self {
        Self {
            vars: HashMap::from([
                (
                    "PS1".into(),
                    Variable {
                        contents: ElviType::String("$ ".into()),
                        ..Default::default()
                    },
                ),
                (
                    "IFS".into(),
                    Variable {
                        contents: ElviType::String(r" \t\n".into()),
                        ..Default::default()
                    },
                ),
                (
                    "PATH".into(),
                    Variable {
                        contents: ElviType::String("/usr/sbin:/usr/bin:/sbin:/bin".into()),
                        ..Default::default()
                    },
                ),
                (
                    "?".into(),
                    Variable {
                        contents: ElviType::ErrExitCode(0),
                        modification_status: ElviMutable::ReadonlyUnsettable,
                        ..Default::default()
                    },
                ),
                (
                    "PWD".into(),
                    Variable {
                        contents: ElviType::String(
                            env::current_dir().unwrap().to_str().unwrap().to_string(),
                        ),
                        ..Default::default()
                    },
                ),
                (
                    "OLDPWD".into(),
                    Variable {
                        contents: ElviType::String(
                            env::current_dir().unwrap().to_str().unwrap().to_string(),
                        ),
                        ..Default::default()
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
                        ..Default::default()
                    },
                ),
                (
                    "ELVI_VERSION".into(),
                    Variable {
                        contents: ElviType::String(env!("CARGO_PKG_VERSION").to_string()),
                        ..Default::default()
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
            contents: ElviType::String(String::new()),
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

impl From<ElviType> for Variable {
    fn from(value: ElviType) -> Self {
        Self {
            contents: value,
            ..Default::default()
        }
    }
}

impl ElviType {
    /// Return an escaped string using [`backslash::escape_ascii`].
    ///
    /// # Panics
    /// Can panic if the string cannot be converted a [`String`] to a UTF-8 byte vector.
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
    /// Requires [`ElviType::BareString`].
    #[must_use]
    pub fn tilde_expansion(&self, vars: &Variables) -> Self {
        match self {
            Self::BareString(le_string) => {
                let path = Path::new(le_string);
                // So in POSIX, you can have two (*three) forms:
                //
                // ```bash
                // ~/foo
                // ~henry/oof
                // ~
                // ```

                // So first let's check if there's even a tilde at the start to speed things up.
                if !le_string.starts_with('~') {
                    return Self::BareString(le_string.to_string());
                }

                if le_string == "~" {
                    return Self::BareString(
                        vars.get_variable("HOME").unwrap().contents.to_string(),
                    );
                // Do we have a tilde starting path?
                } else if path.starts_with("~/") {
                    let mut transform = path.display().to_string();
                    transform.replace_range(
                        0..1,
                        &vars.get_variable("HOME").unwrap().contents.to_string(),
                    );
                    return Self::BareString(transform);
                // At this point, after the previous checks, we can reasonably assume that we are
                // left with a tilde user expansion.
                } else if path.to_str().unwrap().starts_with('~') {
                    let expanded_path: String = match path.parent() {
                        // We have something like `~foo`.
                        Some(p) if p == Path::new("") => {
                            let without_tilde = &path.to_str().unwrap()[1..];
                            let home = get_home(without_tilde).ok().flatten();
                            handle_home(home, &[], &path.display().to_string())
                        }
                        // This means we have the tilde user + paths after
                        Some(x) => {
                            let without_tilde = &x.to_str().unwrap()[1..];
                            let rest: Vec<&OsStr> = path.iter().skip(1).collect();
                            let home = get_home(without_tilde).ok().flatten();
                            handle_home(home, &rest, &path.display().to_string())
                        }
                        None => le_string.to_string(),
                    };
                    return Self::BareString(expanded_path);
                }
                Self::BareString(le_string.to_string())
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
                        chars_of.next();
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
                            chars_of.next();
                            //BUG: What about ${foo\}bar}
                            let tasty_var: String =
                                chars_of.by_ref().take_while(|&c| c != '}').collect();
                            // So here we have a list of expanded special vars, but assuming that
                            // there are no special vars, we got len() == 0.
                            let expanded_out = self.expand_param(&tasty_var, vars);
                            for part in expanded_out {
                                back_to_string.push_str(part.as_str());
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
                            let expanded_out = self.expand_param(&tasty_var, vars);
                            for part in expanded_out {
                                back_to_string.push_str(part.as_str());
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
    pub fn expand_param(&self, var: &str, variables: &Variables) -> Vec<String> {
        let mut ret_vec = vec![];
        match var {
            // TODO: Fully flesh out the differences between these two.
            "*" => {
                if self.is_quoted() {
                    let mut le_shit = vec![];
                    for i in variables.params.iter().skip(1) {
                        le_shit.push(i.contents.to_owned());
                    }
                    return vec![split_ifs(&le_shit, variables)];
                }
            }
            "$" => {
                ret_vec.push(process::id().to_string());
            }
            "#" => ret_vec.push((variables.len_parameters() - 1).to_string()),
            default => {
                if let Some(woot) = variables.get_variable(default) {
                    ret_vec.push(woot.contents.to_string());
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
    ///
    /// # Panics
    /// 1. This can panic if a string returned by [`glob()`] is not valid unicode, which in
    ///    practice will never happen.
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

fn handle_home(home: Option<PathBuf>, rest: &[&OsStr], default: &str) -> String {
    match home {
        Some(mut dir) => {
            for part in rest {
                dir.push(part);
            }
            dir.display().to_string()
        }
        None => default.to_string(),
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

/// Split string according to IFS.
pub fn split_ifs(to_split: &[ElviType], vars: &Variables) -> String {
    let mut reto = vec![];
    let ifs = match vars.get_variable("IFS") {
        Some(yay) => match yay.contents.to_string().chars().next() {
            Some(y) => y.to_string(),
            None => String::new(),
        },
        None => " ".to_string(),
    };
    for part in to_split {
        if !part.is_quoted() {
            reto.push(part);
        }
    }
    reto.iter().join(&ifs)
}
