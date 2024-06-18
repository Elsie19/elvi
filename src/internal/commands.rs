use pest_consume::Itertools;
use std::{
    collections::{hash_map::IntoIter, HashMap},
    fs, mem,
    ops::Deref,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::Command,
};

use super::{
    status::ReturnCode,
    tree::Function,
    variables::{ElviType, Variables},
};

use super::errors::CommandError;

#[derive(Debug, Clone)]
/// Global list of commands, functions, and aliases.
pub struct Commands {
    /// Hashmap of the name of a command, and the path to its executable.
    pub cmds: HashMap<String, PathBuf>,
    /// List of functions.
    pub functions: HashMap<String, Function>,
}

#[derive(Debug, Clone)]
/// Struct to make handling external commands easier.
///
/// According to
/// <https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_09_01>
pub struct ExternalCommand {
    /// Command name.
    pub cmd: PathBuf,
    /// Arguments (if any).
    pub args: Option<Vec<String>>,
    /// Attributes of how a command should be run.
    pub attributes: HowRun,
}

impl Commands {
    /// Generate a list of commands from a path variable.
    ///
    /// # Panics
    /// 1. This will panic if it cannot find the `PATH` variable, which in practice won't happen.
    /// 2. Will panic if `PATH` is not defined as an [`ElviType::String`].
    #[must_use = "Why are you generating PATH if you aren't using it"]
    pub fn generate(variables: &Variables) -> Self {
        let mut cmds: HashMap<String, PathBuf> = HashMap::new();

        let path_var = &variables.get_variable("PATH").unwrap().contents;

        let ElviType::String(path_var) = path_var else {
            unreachable!("How is `PATH` defined as anything but a string? For your debugging information, it is {:?}", path_var)
        };

        for part in path_var.split(':') {
            let Ok(files) = fs::read_dir(part) else {
                continue;
            };

            // Skip all `None`'s.
            for part in files.flatten() {
                cmds.insert(part.file_name().into_string().unwrap(), part.path());
            }
        }
        Self {
            cmds,
            functions: HashMap::new(),
        }
    }

    /// Registers a function and it's contents.
    pub fn register_function(&mut self, func: Function) {
        self.functions.insert(func.name.clone(), func);
    }

    /// Unregister a function.
    pub fn deregister_function(&mut self, name: &str) {
        self.functions.remove(name);
    }

    #[must_use = "Whatcha not doing with this path here bud"]
    pub fn get_path(&self, program: &str) -> Option<PathBuf> {
        self.cmds.get(program).cloned()
    }
}

impl IntoIterator for Commands {
    type Item = (String, PathBuf);

    type IntoIter = IntoIter<String, PathBuf>;

    fn into_iter(self) -> Self::IntoIter {
        self.cmds.into_iter()
    }
}

/// Turn anything that can turn into a str into an [`ExternalCommand`].
impl<T: Deref<Target = str>> From<&T> for ExternalCommand {
    fn from(value: &T) -> Self {
        let value = value as &str;
        let split_up = value.split(' ').collect_vec();
        let cmd = (*split_up.first().unwrap()).to_string();
        if split_up.len() == 1 {
            Self {
                cmd: cmd.into(),
                args: None,
                attributes: HowRun::RealTime,
            }
        } else {
            Self {
                cmd: cmd.into(),
                args: Some(split_up.iter().skip(1).map(|s| (*s).to_string()).collect()),
                attributes: HowRun::RealTime,
            }
        }
    }
}

impl From<Vec<String>> for ExternalCommand {
    fn from(value: Vec<String>) -> Self {
        let cmd = value.first().unwrap().to_owned();
        if value.len() == 1 {
            Self {
                cmd: cmd.into(),
                args: None,
                attributes: HowRun::RealTime,
            }
        } else {
            Self {
                cmd: cmd.into(),
                args: Some(value.iter().skip(1).map(|s| (*s).to_string()).collect()),
                attributes: HowRun::RealTime,
            }
        }
    }
}

/// Contains the output and return code of a command.
pub struct CmdReturn {
    pub ret: ReturnCode,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug, Clone)]
/// Instructs [`ExternalCommand`] how to handle output.
pub enum HowRun {
    /// Run in real time. Use this when you want immediate output, such as when you are not piping
    /// into anything.
    ///
    /// ```bash
    /// ls -la
    /// ```
    RealTime,
    /// Use this when you are doing command substitution. It functions like [`HowRun::Piped`] but
    /// should print stderr out after running.
    ///
    /// ```bash
    /// foo=`ls -la`
    /// ```
    Substitution,
    /// When you are piping into multiple commands.
    ///
    /// ```bash
    /// ls | less
    /// ```
    Piped,
}

impl Default for CmdReturn {
    fn default() -> Self {
        Self {
            ret: ReturnCode::SUCCESS.into(),
            stderr: vec![],
            stdout: vec![],
        }
    }
}

/// Return an almost prepared [`std::process::Command`] where the caller can provide the specifics.
///
/// # Notes
/// Everything should be eval'ed and expanded before using this function.
///
/// # Errors
/// * Will return [`CommandError::NotFound`] if the program does not exist.
/// * Will return [`CommandError::PermissionDenied`] if the path is not executable.
///
/// # Panics
/// Will panic if this scenario happens:
/// 1. Path is absolute or starts with `./`.
/// 2. Path exists.
/// 3. Is not a directory.
/// 4. Cannot access metadata on file.
pub fn execute_external_command(
    cmd: ExternalCommand,
    variables: &Variables,
    commands: &Commands,
) -> Result<std::process::Command, CommandError> {
    #[allow(unused_assignments)]
    let mut cmd_to_run = PathBuf::new();
    // First of all, we have 3 choices of how a command can be run:
    // 1. `foo` with PATH
    // 2. `./foo` | `~tilde/foo` locally
    // 3. `/bin/foo` with qualified path
    // We accomplish this with
    // <https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.is_absolute>

    // We can skip all the PATH checking if the user passed a qualified path.
    //BUG: Does not handle tilde paths yet.
    if cmd.cmd.is_absolute() || cmd.cmd.starts_with("./") {
        if !cmd.cmd.exists() {
            return Err(CommandError::NotFound {
                name: cmd.cmd.display().to_string(),
            });
        // Is some silly goose trying to execute a directory or is not executable.
        } else if cmd.cmd.is_dir() || cmd.cmd.metadata().unwrap().permissions().mode() & 0o111 == 0
        {
            return Err(CommandError::PermissionDenied {
                path: cmd.cmd.display().to_string(),
            });
        }

        cmd_to_run = cmd.cmd;
    // This means we have a normal path that we need PATH to get
    } else {
        cmd_to_run = if let Some(v) = commands.get_path(&cmd.cmd.display().to_string()) {
            v
        } else {
            return Err(CommandError::NotFound {
                name: cmd.cmd.display().to_string(),
            });
        };
    }

    // Now that we have our path to the binary, let's get rolling on running it.
    // Firstly, we need to prepare our environmental variables.
    let filtered_env = variables.get_environmentals();
    // Huzzah! Now this is where the magic happens, and it is very confusing, but basically:
    //
    // 1. Create command that takes our full path because we have already calculated it by PATH.
    // 2. Clear environment.
    // 3. Insert our own.
    // 4. Set current directory based on PWD.
    let bitch = Command::new("");
    let mut binding = Command::new(cmd_to_run);
    let bruh = binding
        .args(cmd.args.unwrap_or_else(Vec::new))
        .env_clear()
        .current_dir(variables.get_variable("PWD").unwrap().contents.to_string())
        .envs(filtered_env);
    // So this was the only way I could figure out how to swap mutability and satisfy the borrow
    // checker.
    Ok(mem::replace(bruh, bitch))
}
