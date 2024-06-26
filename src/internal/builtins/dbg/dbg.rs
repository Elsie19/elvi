use getopts::Options;

use crate::internal::errors::{ElviError, VariableError};
use crate::internal::status::ReturnCode;
use crate::internal::variables::{ElviMutable, ElviType, Variables};

/// The internal code that runs when the `dbg` builtin is run.
pub fn main(args: Option<&[ElviType]>, variables: &mut Variables) -> ReturnCode {
    let mut opts = Options::new();
    let mut evaled_variables = vec![];
    opts.optflag("h", "help", "print help menu");

    if let Some(unny) = args {
        for part in unny {
            evaled_variables.push(
                part.tilde_expansion(variables)
                    .eval_variables(variables)
                    .eval_escapes()
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
        print_usage("dbg", &opts);
        return ReturnCode::SUCCESS.into();
    }

    if matches.free.is_empty() {
        print_usage("dbg", &opts);
        return ReturnCode::FAILURE.into();
    }

    let Some(var) = variables.get_variable(&matches.free[0]) else {
        let err = VariableError::NoSuchVariable {
            name: matches.free[0].to_string(),
            caller: "dbg",
        };
        eprintln!("{err}");
        return err.ret();
    };

    match var.modification_status {
        ElviMutable::Normal => println!("{}={:?}", matches.free[0], var.contents),
        ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
            println!("readonly {}={}", matches.free[0], var.contents);
        }
    }
    ReturnCode::SUCCESS.into()
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {program} VARNAME");
    print!("{}", opts.usage(&brief));
}
