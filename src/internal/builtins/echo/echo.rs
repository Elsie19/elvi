use crate::internal::status::ReturnCode;
use crate::internal::variables::{ElviType, Variables};
use getopts::Options;

/// The internal code that runs when the `echo` builtin is run.
#[must_use]
pub fn main(text: Option<&[ElviType]>, variables: &Variables) -> ReturnCode {
    let mut opts = Options::new();
    let mut evaled_variables = vec![];
    opts.optflag("n", "", "do not append a newline");

    if let Some(unny) = text {
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

    let mut to_print = vec![];
    if matches.free.is_empty() {
        if matches.opt_present("n") {
            print!("");
        } else {
            println!();
        }
        ReturnCode::SUCCESS.into()
    } else {
        for part in &matches.free {
            to_print.push(part.to_owned().replace("\\033", "\x1b"));
        }
        if matches.opt_present("n") {
            print!("{}", to_print.join(" "));
        } else {
            println!("{}", to_print.join(" "));
        }
        ReturnCode::SUCCESS.into()
    }
}
