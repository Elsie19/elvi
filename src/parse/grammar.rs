use std::io::{self, Write};
use std::process::Command;

use crate::internal::builtins;
use crate::internal::commands::{CommandError, Commands, ExternalCommand};
use crate::internal::status::ReturnCode;
use crate::internal::tree::{Actions, Builtins};
use crate::internal::variables::{ElviGlobal, ElviMutable, ElviType, Variable, Variables};
use pest_consume::{match_nodes, Error, Parser};

#[derive(Parser)]
#[grammar = "parse/internals/strings.pest"]
#[grammar = "parse/internals/variables.pest"]
#[grammar = "parse/internals/command_substitution.pest"]
#[grammar = "parse/internals/builtins.pest"]
#[grammar = "parse/internals/commands.pest"]
#[grammar = "parse/internals/if.pest"]
#[grammar = "parse/internals/base.pest"]
/// Global struct that implements the pest.rs parser ([`pest_derive`]).
pub struct ElviParser;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

// This is the other half of the parser, using pest_consume.
#[pest_consume::parser]
impl ElviParser {
    #[allow(clippy::used_underscore_binding)]
    /// Handles end of file.
    pub fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    /// Handles any number.
    pub fn elviNumber(input: Node) -> Result<u16> {
        Ok(input.as_str().parse().unwrap())
    }

    /// Handles a variable name.
    pub fn variableIdent(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    pub fn doubleInner(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    pub fn singleInner(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    /// Handles single quotes.
    pub fn singleQuoteString(input: Node) -> Result<ElviType> {
        Ok(match_nodes!(input.into_children();
            [singleInner(stringo)] => ElviType::String(stringo),
        ))
    }

    /// Handles double quotes.
    pub fn doubleQuoteString(input: Node) -> Result<ElviType> {
        Ok(match_nodes!(input.into_children();
            [doubleInner(stringo)] => ElviType::String(stringo).eval_escapes(),
        ))
    }

    pub fn backtickInner(input: Node) -> Result<ElviType> {
        Ok(ElviType::CommandSubstitution(input.as_str().to_string()))
    }

    /// Handles backtick substitution.
    pub fn backtickSubstitution(input: Node) -> Result<ElviType> {
        Ok(match_nodes!(input.into_children();
            [backtickInner(stringo)] => stringo,
        ))
    }

    /// Wrapper to handle any valid string.
    pub fn anyString(input: Node) -> Result<ElviType> {
        Ok(match_nodes!(input.into_children();
            [singleQuoteString(stringo)] => stringo,
            [doubleQuoteString(stringo)] => stringo,
        ))
    }

    /// Wrapper to handle any valid assignment of a variable.
    pub fn variableIdentifierPossibilities(input: Node) -> Result<ElviType> {
        Ok(match_nodes!(input.into_children();
            [anyString(stringo)] => stringo,
            [backtickSubstitution(stringo)] => stringo,
        ))
    }

    /// Handles normal variable assignments.
    pub fn normalVariable(input: Node) -> Result<(String, Variable)> {
        let mut stuff = input.clone().into_children().into_pairs();

        let lines = stuff.clone().next().unwrap().line_col();

        let name_pair = stuff.next().unwrap().as_str();

        let variable_contents =
            Self::variableIdentifierPossibilities(input.clone().into_children().nth(1).unwrap());

        Ok((
            name_pair.to_string(),
            Variable::oneshot_var(
                variable_contents.unwrap(),
                ElviMutable::Normal,
                ElviGlobal::Normal(1),
                lines,
            ),
        ))
    }

    /// Handles readonly variable assignments.
    pub fn readonlyVariable(input: Node) -> Result<(String, Variable)> {
        let mut stuff = input.clone().into_children().into_pairs();

        let lines = stuff.clone().next().unwrap().line_col();

        let name_pair = stuff.next().unwrap().as_str();

        let variable_contents =
            Self::variableIdentifierPossibilities(input.clone().into_children().nth(1).unwrap());

        Ok((
            name_pair.to_string(),
            Variable::oneshot_var(
                variable_contents.unwrap(),
                ElviMutable::Readonly,
                ElviGlobal::Normal(1),
                lines,
            ),
        ))
    }

    /// Handles the readonly builtin.
    pub fn builtinDbg(input: Node) -> Result<Actions> {
        let name = input
            .into_children()
            .into_pairs()
            .next()
            .unwrap()
            .as_str()
            .to_string();

        Ok(Actions::Builtin(Builtins::Dbg(name)))
    }

    /// Handles the unset builtin.
    pub fn builtinUnset(input: Node) -> Result<Actions> {
        let name = input
            .into_children()
            .into_pairs()
            .next()
            .unwrap()
            .as_str()
            .to_string();

        Ok(Actions::Builtin(Builtins::Unset(name)))
    }

    /// Handles the exit builtin.
    pub fn builtinExit(input: Node) -> Result<Actions> {
        let possibles = match_nodes!(input.into_children();
            [anyString(stringo)] => Some(stringo),
            [elviNumber(stringo)] => Some(ElviType::String(stringo.to_string())),
            [] => None,
        );

        Ok(Actions::Builtin(Builtins::Exit(possibles)))
    }

    /// Handles any external command.
    pub fn externalCommand(input: Node) -> Result<Actions> {
        Ok(Actions::Command(vec!["dbgbar".to_string()]))
    }

    /// Handles if statements.
    pub fn ifStatement(input: Node) -> Result<()> {
        Ok(())
    }

    /// Handles global statements.
    pub fn statement(input: Node) -> Result<Actions> {
        match_nodes!(input.into_children();
            [normalVariable(var)] => {
                Ok(Actions::ChangeVariable(var))
            },
            [readonlyVariable(var)] => {
                Ok(Actions::ChangeVariable(var))
            },
            [builtinDbg(var)] => Ok(var),
            [builtinExit(var)] => Ok(var),
            [builtinUnset(var)] => Ok(var),
            // [externalCommand(var)] => Ok(var),
            // [ifStatement(var)] => Ok(var),
        )
    }

    /// Entry point for parsing.
    pub fn program(input: Node) -> ReturnCode {
        let mut variables = Variables::default();
        let mut commands = Commands::generate(&variables);

        let mut subshells_in = 1;

        for child in input.into_children() {
            if child.as_rule() != Rule::EOI {
                match Self::statement(child) {
                    Ok(yes) => match yes {
                        Actions::ChangeVariable((name, var)) => match var.get_value() {
                            // If we have a string we can just assign as is, as the escaping has
                            // already been done.
                            ElviType::String(_x) => {
                                if var.get_lvl() != ElviGlobal::Global {
                                    let mut var = var.clone();
                                    var.change_lvl(subshells_in);
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
                                        let mut var = var.clone();
                                        var.change_lvl(subshells_in);
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
                            _ => unimplemented!("Is not done yet"),
                        },
                        Actions::Builtin(built) => match built {
                            Builtins::Dbg(var) => {
                                let ret = builtins::dbg::builtin_dbg(&var, &mut variables).get();
                                variables.set_ret(ReturnCode::ret(ret));
                            }
                            Builtins::Exit(var) => {
                                let ret = builtins::exit::builtin_exit(var);
                                if subshells_in > 1 {
                                    subshells_in -= 1;
                                } else {
                                    std::process::exit(ret.get().into());
                                }
                            }
                            Builtins::Unset(var) => {
                                let ret =
                                    builtins::unset::builtin_unset(&var, &mut variables).get();
                                variables.set_ret(ReturnCode::ret(ret));
                            }
                        },
                        Actions::Command(cmd) => {
                            println!("Running command {cmd:?}");
                        }
                        Actions::Null => {}
                    },
                    Err(oops) => {
                        eprintln!("{oops}");
                        continue;
                    }
                }
            }
        }

        let ret_value = match variables.get_variable("?").unwrap().get_value() {
            ElviType::ErrExitCode(x) => *x,
            _ => unreachable!("How is $? defined as anything but ErrExitCode?????"),
        };

        ReturnCode::ret(ret_value)
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::{ElviParser, Rule};

    #[test]
    fn double_quote_string_is_chill() {
        let stringo = r#""foobar""#;
        let parse = ElviParser::parse(Rule::doubleQuoteString, stringo).unwrap();
        assert_eq!(r#""foobar""#, parse.as_str());
    }
}
