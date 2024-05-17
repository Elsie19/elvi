use std::io::Write;
use std::{io, process::Command};

use crate::internal::commands::CommandError;

use super::{
    commands::{Commands, ExternalCommand},
    variables::{ElviGlobal, ElviType, Variable, Variables},
};

#[derive(Debug)]
/// A list of possible actions a line can cause.
pub enum Actions {
    ChangeVariable((String, Variable)),
    Builtin(Builtins),
    Command(Vec<String>),
    Null,
}

#[derive(Debug)]
/// A list of builtins and their parameters.
pub enum Builtins {
    Dbg(String),
    Unset(String),
    Exit(Option<ElviType>),
}

pub fn change_variable(
    variables: &mut Variables,
    commands: &Commands,
    lvl: u32,
    name: String,
    var: Variable,
) {
    // Makes shit easier to deal with.
    let mut var = var.clone();
    match var.get_value() {
        ElviType::String(_we_dont_care) => {
            // First let's get the level because while parsing we assume a certain variable level that is
            // not true to the actual evaluating.
            if var.get_lvl() != ElviGlobal::Global {
                var.change_lvl(lvl);
            }
            match variables.set_variable(name, var) {
                Ok(()) => {}
                Err(oops) => eprintln!("{oops}"),
            }
        }
        ElviType::CommandSubstitution(x) => {
            //TODO: Interpolate the variables if any
            let cmd_to_run = ExternalCommand::string_to_command(x);
            // Set variable to empty if we can't get the command
            if commands.get_path(&cmd_to_run.cmd).is_none() {
                eprintln!(
                    "{}",
                    CommandError::NotFound {
                        name: cmd_to_run.cmd,
                    }
                );
                if var.get_lvl() != ElviGlobal::Global {
                    var.change_lvl(lvl);
                }
                match variables.set_variable(
                    name,
                    Variable::oneshot_var(
                        ElviType::String(String::new()),
                        var.get_modification_status(),
                        var.get_lvl(),
                        var.get_line(),
                    ),
                ) {
                    Ok(()) => {}
                    Err(oops) => eprintln!("{oops}"),
                }
            } else {
                // Let's get our environmental variables ready
                let filtered_env = variables.get_environmentals();
                // Full path, this is important as we don't want Command::new()
                // to be fooling around with it's own PATH, even though we
                // override it.
                let patho = commands.get_path(&cmd_to_run.cmd).unwrap();
                let cmd = Command::new(patho.to_str().unwrap())
                    .args(if cmd_to_run.args.is_none() {
                        vec![]
                    } else {
                        cmd_to_run.args.unwrap()
                    })
                    .env_clear()
                    .current_dir(
                        variables
                            .get_variable("PWD")
                            .unwrap()
                            .get_value()
                            .to_string(),
                    )
                    .envs(filtered_env)
                    .output()
                    .expect("oops");

                if !cmd.stderr.is_empty() {
                    io::stderr().write_all(&cmd.stderr).unwrap();
                }
                let mut var = var.clone();
                match variables.set_variable(
                    name,
                    var.change_contents(ElviType::String(
                        std::str::from_utf8(&cmd.stdout).unwrap().to_string(),
                    ))
                    .clone(),
                ) {
                    Ok(()) => {}
                    Err(oops) => eprintln!("{oops}"),
                }
            }
        }
        _ => unimplemented!("Give me a break please"),
    }
}
