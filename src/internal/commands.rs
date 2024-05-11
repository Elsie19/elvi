use std::{collections::HashMap, fs, path::PathBuf};

use super::variables::{ElviType, Variables};

#[derive(Debug)]
pub struct Commands {
    cmds: HashMap<String, PathBuf>,
}

impl Commands {
    pub fn generate(variables: &Variables) -> Self {
        let mut cmds: HashMap<String, PathBuf> = HashMap::new();

        let path_var = variables.get_variable("PATH".into()).unwrap().get_value();

        let path_var = match path_var {
            ElviType::String(foo) => foo,
            _ => unreachable!("How is `PATH` defined as anything but a string?"),
        };

        for part in path_var.split(":") {
            let files = match fs::read_dir(part) {
                Ok(yas) => yas,
                // Skip over it if the path don't exist
                Err(_) => continue,
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

    pub fn get_path(&self, program: String) -> PathBuf {
        self.cmds.get(&program).unwrap().to_path_buf()
    }
}
