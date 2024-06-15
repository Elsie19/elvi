use getopts::Options;

use crate::internal::commands::Commands;
use crate::internal::errors::VariableError;
use crate::internal::status::ReturnCode;
use crate::internal::variables::ElviMutable;
use crate::internal::variables::ElviType;
use crate::internal::variables::Variables;

#[derive(PartialEq)]
enum TypeUnset {
    Function,
    Variable,
}

/// The internal code that runs when the `unset` builtin is run.
#[allow(clippy::module_name_repetitions)]
pub fn builtin_unset(
    args: Option<&[ElviType]>,
    variables: &mut Variables,
    commands: &mut Commands,
) -> ReturnCode {
    let mut opts = Options::new();
    let mut to_unset = TypeUnset::Variable;
    let mut evaled_variables = vec![];
    opts.optflag("h", "help", "print help menu");
    opts.optflag("v", "", "treat each NAME as a shell variable");
    opts.optflag("f", "", "treat each NAME as a shell function");

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
        print_usage("unset", &opts);
        return ReturnCode::SUCCESS.into();
    }
    if matches.opt_present("f") {
        to_unset = TypeUnset::Function;
    } else if matches.opt_present("v") {
        to_unset = TypeUnset::Variable;
    }
    if matches.free.is_empty() {
        print_usage("unset", &opts);
        return ReturnCode::MISUSE.into();
    }

    let mut return_code: ReturnCode = ReturnCode::SUCCESS.into();

    for name in matches.free {
        if to_unset == TypeUnset::Function {
            commands.deregister_function(&name);
        } else {
            let Some(var) = variables.get_variable(&name) else {
        // <https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/utilities/V3_chap02.html#unset> in description in 5th paragraph
        return ReturnCode::SUCCESS.into();
    };
            match var.modification_status {
                ElviMutable::Normal => match variables.unset(&name) {
                    // We don't care about what it returned
                    Some(()) | None => {}
                },
                ElviMutable::Readonly | ElviMutable::ReadonlyUnsettable => {
                    let err = VariableError::Readonly {
                        name: "unset".to_string(),
                        lines: var.line,
                    };
                    eprintln!("{err}");
                    return_code = ReturnCode::FAILURE.into();
                }
            }
        }
    }
    return_code
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {program} [-fv] [name ...]");
    print!("{}", opts.usage(&brief));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::variables::Variable;

    #[test]
    #[should_panic]
    fn did_unset() {
        let mut variables = Variables::default();
        let mut commands = Commands::generate(&variables);
        variables
            .set_variable(
                "foo",
                Variable {
                    contents: ElviType::String("bar".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        builtin_unset(
            Some(&[ElviType::String("foo".to_string())]),
            &mut variables,
            &mut commands,
        );
        variables.get_variable("foo").unwrap();
    }

    #[test]
    fn cannot_unset_readonly() {
        let mut variables = Variables::default();
        let mut commands = Commands::generate(&variables);
        variables
            .set_variable(
                "foo",
                Variable {
                    contents: ElviType::String("bar".to_string()),
                    modification_status: ElviMutable::Readonly,
                    ..Default::default()
                },
            )
            .unwrap();
        let out = builtin_unset(
            Some(&[ElviType::String("foo".to_string())]),
            &mut variables,
            &mut commands,
        );
        assert_eq!(out, ReturnCode::FAILURE.into());
    }
}
