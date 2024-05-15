use custom_error::custom_error;
use pest_consume::Itertools;
use std::{collections::HashMap, fs, path::PathBuf};

use super::variables::{ElviType, Variables};

custom_error! {pub CommandError
    NotFound{name:String} = "elvi: {name}: not found",
}

#[derive(Debug, Clone)]
/// Global list of commands.
pub struct Commands {
    /// Hashmap of the name of a command, and the path to its executable.
    cmds: HashMap<String, PathBuf>,
}

#[derive(Debug, Clone)]
/// Struct to make handling external commands easier
pub struct ExternalCommand {
    pub cmd: String,
    pub args: Option<Vec<String>>,
}

impl Commands {
    /// Generate a list of commands from a path variable.
    pub fn generate(variables: &Variables) -> Self {
        let mut cmds: HashMap<String, PathBuf> = HashMap::new();

        let path_var = variables.get_variable("PATH").unwrap().get_value();

        let ElviType::String(path_var) = path_var else {
            unreachable!("How is `PATH` defined as anything but a string?")
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

impl ExternalCommand {
    pub fn string_to_command(cmd: String) -> Self {
        let split_up = cmd.split(" ").collect_vec();
        let cmd = split_up.get(0).unwrap().to_string();
        if split_up.len() > 1 {
            Self { cmd, args: None }
        } else {
            Self {
                cmd,
                args: Some(split_up.iter().skip(1).map(|s| s.to_string()).collect()),
            }
        }
    }
}
