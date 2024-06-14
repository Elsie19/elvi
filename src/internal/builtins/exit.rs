use getopts::Options;

use crate::internal::{
    errors::{ElviError, VariableError},
    status::ReturnCode,
    variables::{ElviType, Variables},
};

/// The internal code that runs when the `exit` builtin is run.
///
/// This builtin corresponds with
/// <https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/utilities/V3_chap02.html#exit>
/// except for trapping.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn builtin_exit(args: Option<&[ElviType]>, variables: &Variables) -> ReturnCode {
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
        print_usage("exit", &opts);
        return ReturnCode::SUCCESS.into();
    }

    match matches.free.first() {
        Some(yo) => {
            let try_code = yo.to_string().parse::<ReturnCode>();
            if let Ok(yay) = try_code {
                yay
            } else {
                let err = VariableError::IllegalNumber {
                    name: yo.to_string(),
                    caller: "exit",
                };
                eprintln!("{err}");
                err.ret()
            }
        }
        // If we have a bare exit, return with 0.
        None => ReturnCode::SUCCESS.into(),
    }
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {program} [n]");
    print!("{}", opts.usage(&brief));
}
