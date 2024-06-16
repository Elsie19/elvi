use getopts::Options;

use crate::internal::errors::{ElviError, VariableError};
use crate::internal::status::ReturnCode;
use crate::internal::variables::{ElviType, Variables};

/// The internal code that runs when the `shift` builtin is run.
pub fn main(args: Option<&[ElviType]>, variables: &mut Variables) -> ReturnCode {
    let mut opts = Options::new();
    let mut evaled_variables = vec![];
    opts.optflag("h", "help", "print help message");

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
        print_usage("shift", &opts);
        return ReturnCode::SUCCESS.into();
    }
    if matches.free.is_empty() {
        return ReturnCode::SUCCESS.into();
    }
    let number = matches.free[0].clone();
    let parsed = number.parse::<usize>();
    let Ok(number) = parsed else {
        let err = VariableError::IllegalNumber {
            name: number,
            caller: "shift",
        };
        eprintln!("{err}");
        return err.ret();
    };
    if variables.params.len() < number {
        eprintln!("can't shift that many");
        std::process::exit(ReturnCode::MISUSE.into());
    }
    variables.params.drain(..number);
    ReturnCode::SUCCESS.into()
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {program} [n]");
    print!("{}", opts.usage(&brief));
}
