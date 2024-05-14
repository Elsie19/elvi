use crate::internal::builtins::{self, *};
use crate::internal::commands::Commands;
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
pub struct ElviParser;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

// This is the other half of the parser, using pest_consume.
#[pest_consume::parser]
impl ElviParser {
    pub fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    pub fn variableIdent(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    pub fn singleQuoteString(input: Node) -> Result<ElviType> {
        Ok(ElviType::String(input.as_str().to_string()))
    }

    //TODO: Variable interpolation
    pub fn doubleQuoteString(input: Node) -> Result<ElviType> {
        Ok(ElviType::String(input.as_str().to_string())
            .eval_escapes()
            .unwrap())
    }

    //TODO: Command substitution
    pub fn backtickSubstitution(input: Node) -> Result<ElviType> {
        Ok(ElviType::String(input.as_str().to_string()))
    }

    pub fn anyString(input: Node) -> Result<ElviType> {
        Ok(match_nodes!(input.into_children();
            [singleQuoteString(stringo)] => stringo,
            [doubleQuoteString(stringo)] => stringo,
        ))
    }

    pub fn variableIdentifierPossibilities(input: Node) -> Result<ElviType> {
        Ok(match_nodes!(input.into_children();
            [anyString(stringo)] => stringo,
            [backtickSubstitution(stringo)] => stringo,
        ))
    }

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

    pub fn externalCommand(input: Node) -> Result<Actions> {
        Ok(Actions::Command(vec!["dbgbar".to_string()]))
    }

    pub fn ifStatement(input: Node) -> Result<()> {
        Ok(())
    }

    pub fn statement(input: Node) -> Result<Actions> {
        match_nodes!(input.into_children();
            [normalVariable(var)] => {
                Ok(Actions::ChangeVariable(var))
            },
            [readonlyVariable(var)] => {
                Ok(Actions::ChangeVariable(var))
            },
            [builtinDbg(var)] => Ok(var),
            // [externalCommand(var)] => Ok(var),
            // [ifStatement(var)] => Ok(var),
        )
    }

    pub fn program(input: Node) -> Result<u8> {
        let mut variables = Variables::default();
        let mut commands = Commands::generate(&variables);

        let mut subshells_in = 1;

        for child in input.into_children() {
            if child.as_rule() != Rule::EOI {
                match Self::statement(child) {
                    Ok(yes) => match yes {
                        Actions::ChangeVariable((name, var)) => {
                            if var.get_lvl() != ElviGlobal::Global {
                                let mut var = var.clone();
                                var.change_lvl(subshells_in);
                            }
                            // println!("Changing variable '{}' with contents of: {:?}", name, var);
                            match variables.set_variable(name, var) {
                                Ok(_) => {}
                                Err(foo) => eprintln!("{foo}"),
                            }
                        }
                        Actions::Builtin(built) => match built {
                            Builtins::Dbg(var) => {
                                if builtins::dbg::builtin_dbg(var.clone(), &mut variables).get()
                                    != ReturnCode::SUCCESS
                                {
                                    variables.set_ret(ReturnCode::ret(1));
                                } else {
                                    variables.set_ret(ReturnCode::ret(0));
                                }
                            }
                        },
                        Actions::Command(cmd) => {
                            println!("Running command {:?}", cmd);
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

        let ret_value = match variables.get_variable("?".into()).unwrap().get_value() {
            ElviType::ErrExitCode(x) => *x,
            _ => unreachable!("How is $? defined as anything but ErrExitCode?????"),
        };

        Ok(ret_value)
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
