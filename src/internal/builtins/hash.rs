use getopts::Options;

use crate::internal::commands::Commands;
use crate::internal::status::ReturnCode;
use crate::internal::variables::{ElviType, Variables};

/// The internal code that runs when the `hash` builtin is run.
#[allow(clippy::module_name_repetitions)]
pub fn builtin_hash(
    args: Option<&[ElviType]>,
    commands: &mut Commands,
    variables: &Variables,
) -> ReturnCode {
    let mut opts = Options::new();
    let mut evaled_variables = vec![];
    opts.optflag("r", "", "forget all remembered locations");

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

    if matches.opt_present("r") {
        *commands = Commands::generate(variables);
    } else if matches.free.is_empty() {
        for (cmd, patho) in &commands.cmds {
            println!("{}={}", cmd, patho.display());
        }
    } else {
        print_usage("hash", opts);
        return ReturnCode::MISUSE.into();
    }

    ReturnCode::SUCCESS.into()
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [-r] [name ...]", program);
    print!("{}", opts.usage(&brief));
}
