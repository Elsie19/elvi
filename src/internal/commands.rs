use std::{collections::HashMap, fs, path::PathBuf};

use super::variables::{ElviType, Variables};

#[derive(Debug)]
pub struct Commands {
    cmds: HashMap<String, PathBuf>,
}

impl Commands {
    pub fn generate(variables: &Variables) -> Self {
        let mut cmds: HashMap<String, PathBuf> = HashMap::new();

        let path_var = variables.get_variable("PATH").unwrap().get_value();

        let ElviType::String(path_var) = path_var else { unreachable!("How is `PATH` defined as anything but a string?") };

        for part in path_var.split(':') {
            let Ok(files) = fs::read_dir(part) else { continue };

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

    pub fn get_path(&self, program: &str) -> PathBuf {
        self.cmds.get(program).unwrap().to_path_buf()
    }
}
