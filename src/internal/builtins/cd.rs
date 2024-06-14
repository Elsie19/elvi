use std::path::PathBuf;
use std::{env, fs};

use getopts::Options;

use crate::internal::errors::{CommandError, ElviError};
use crate::internal::status::ReturnCode;
use crate::internal::variables::{ElviType, Variable, Variables};

/// The internal code that runs when the `cd` builtin is run.
///
/// # Todo
/// Fix this spaghetti code.
#[allow(clippy::module_name_repetitions)]
#[allow(clippy::too_many_lines)]
pub fn builtin_cd(args: Option<&[ElviType]>, variables: &mut Variables) -> ReturnCode {
    let mut opts = Options::new();
    let mut evaled_variables = vec![];
    opts.optflag("h", "help", "print help menu");

    if let Some(unny) = args {
        for part in unny {
            evaled_variables.push(
                part.tilde_expansion(variables)
                    .eval_escapes()
                    .eval_variables(variables)
                    .to_string(),
            );
        }
    }

    let matches = match opts.parse(evaled_variables) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{f}");
            return ReturnCode::MISUSE.into();
        }
    };
    if matches.opt_present("h") {
        print_usage("cd", &opts);
        return ReturnCode::SUCCESS.into();
    }

    if matches.free.is_empty() {
        match variables.set_variable(
            "OLDPWD".to_string(),
            variables.get_variable("PWD").unwrap().to_owned(),
        ) {
            Ok(()) => {}
            Err(oops) => eprintln!("{oops}"),
        }
        match variables.set_variable(
            "PWD".to_string(),
            variables.get_variable("HOME").unwrap().clone(),
        ) {
            Ok(()) => {}
            Err(oops) => eprintln!("{oops}"),
        }
        assert!(env::set_current_dir(
            variables
                .get_variable("HOME")
                .unwrap()
                .get_value()
                .to_string()
        )
        .is_ok());
        return ReturnCode::SUCCESS.into();
    }
    // Atp we know we got something, so let's check if it's `-` or a path.
    match matches.free[0].as_str() {
        "-" => {
            let swap = variables.get_variable("PWD").unwrap().clone();
            match variables.set_variable(
                "PWD".to_string(),
                variables.get_variable("OLDPWD").unwrap().clone(),
            ) {
                Ok(()) => {}
                Err(oops) => eprintln!("{oops}"),
            }
            println!("{}", variables.get_variable("PWD").unwrap().get_value());
            match variables.set_variable("OLDPWD".to_string(), swap) {
                Ok(()) => {}
                Err(oops) => eprintln!("{oops}"),
            }
            assert!(env::set_current_dir(
                variables
                    .get_variable("PWD")
                    .unwrap()
                    .get_value()
                    .to_string()
            )
            .is_ok());
            ReturnCode::SUCCESS.into()
        }
        patho => {
            let to_cd = PathBuf::from(
                ElviType::String(patho.to_string())
                    .tilde_expansion(variables)
                    .to_string(),
            );
            if !to_cd.exists() {
                let err = CommandError::CannotCd {
                    name: "cd".to_string(),
                    path: to_cd.to_str().unwrap().to_string(),
                };
                eprintln!("{err}");
                return err.ret();
            }
            if fs::read_dir(to_cd.to_str().unwrap()).is_err() {
                let err = CommandError::CannotCd {
                    name: "cd".to_string(),
                    path: to_cd.to_str().unwrap().to_string(),
                };
                eprintln!("{err}");
                return err.ret();
            }
            // Ok so the path exists, time to roll.
            match variables.set_variable(
                "OLDPWD".to_string(),
                variables.get_variable("PWD").unwrap().clone(),
            ) {
                Ok(()) => {}
                Err(oops) => eprintln!("{oops}"),
            }
            match variables.set_variable(
                "PWD".to_string(),
                Variable::oneshot_var(
                    &ElviType::String(to_cd.to_str().unwrap().to_string()),
                    crate::internal::variables::ElviMutable::Normal,
                    crate::internal::variables::ElviGlobal::Global,
                    (0, 0),
                ),
            ) {
                Ok(()) => {}
                Err(oops) => eprintln!("{oops}"),
            }
            assert!(env::set_current_dir(to_cd).is_ok());
            ReturnCode::SUCCESS.into()
        }
    }
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {program} PATH");
    print!("{}", opts.usage(&brief));
}
