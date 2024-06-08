use custom_error::custom_error;
use pest_consume::Itertools;
use std::{
    collections::{hash_map::IntoIter, HashMap},
    fs,
    path::PathBuf,
};

use super::variables::{ElviType, Variables};

custom_error! { pub CommandError
    NotFound {name: String} = "elvi: {name}: not found",
    SubCommandNotFound {name: &'static str, cmd: String} = "elvi: {name}: {cmd}: not found",
    CannotCd {name: String, path: String} = "elvi: {name}: can't cd to {path}",
}

#[derive(Debug, Clone)]
/// Global list of commands.
pub struct Commands {
    /// Hashmap of the name of a command, and the path to its executable.
    cmds: HashMap<String, PathBuf>,
}

#[derive(Debug, Clone)]
/// Struct to make handling external commands easier.
pub struct ExternalCommand {
    /// Command name.
    pub cmd: String,
    /// Arguments (if any).
    pub args: Option<Vec<String>>,
}

impl From<&String> for ExternalCommand {
    fn from(value: &String) -> Self {
        Self::into(value.into())
    }
}

impl Commands {
    /// Generate a list of commands from a path variable.
    pub fn generate(variables: &Variables) -> Self {
        let mut cmds: HashMap<String, PathBuf> = HashMap::new();

        let path_var = variables.get_variable("PATH").unwrap().get_value();

        let ElviType::String(path_var) = path_var else {
            unreachable!("How is `PATH` defined as anything but a string? For your debugging information, it is {:?}", path_var)
        };

        for part in path_var.split(':') {
            let Ok(files) = fs::read_dir(part) else {
                continue;
            };

            for part in files {
                if part.as_ref().unwrap().path().is_file() {
                    cmds.insert(
                        part.as_ref().unwrap().file_name().into_string().unwrap(),
                        part.unwrap().path(),
                    );
                }
            }
        }
        Self { cmds }
    }

    pub fn get_path(&self, program: &str) -> Option<PathBuf> {
        self.cmds.get(program).map(|cmd| cmd.to_path_buf())
    }
}

impl IntoIterator for Commands {
    type Item = (String, PathBuf);

    type IntoIter = IntoIter<String, PathBuf>;

    fn into_iter(self) -> Self::IntoIter {
        self.cmds.into_iter()
    }
}

impl From<&str> for ExternalCommand {
    fn from(value: &str) -> Self {
        let split_up = value.split(' ').collect_vec();
        let cmd = (*split_up.first().unwrap()).to_string();
        if split_up.len() == 1 {
            Self { cmd, args: None }
        } else {
            Self {
                cmd,
                args: Some(split_up.iter().skip(1).map(|s| (*s).to_string()).collect()),
            }
        }
    }
}
