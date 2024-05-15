use std::{collections::HashMap, fs, path::PathBuf};

use super::variables::{ElviType, Variables};

#[derive(Debug, Clone)]
/// Global list of commands.
pub struct Commands {
    /// Hashmap of the name of a command, and the path to its executable.
    cmds: HashMap<String, PathBuf>,
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
