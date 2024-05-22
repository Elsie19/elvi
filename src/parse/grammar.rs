use crate::internal::builtins;
use crate::internal::commands::Commands;
use crate::internal::status::ReturnCode;
use crate::internal::tree::{change_variable, Actions, Builtins, TestOptions};
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
            [doubleInner(stringo)] => ElviType::VariableSubstitution(stringo),
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

    pub fn builtinTestPrimaries(input: Node) -> Result<TestOptions> {
        Ok(match_nodes!(input.into_children();
            [block # elviWord(stringo)] => TestOptions::BlockFileExists(stringo),
            [character_special # elviWord(stringo)] => TestOptions::CharacterFileExists(stringo),
            [directory_exists # elviWord(stringo)] => TestOptions::DirectoryExists(stringo),
            [file_exists # elviWord(stringo)] => TestOptions::AnyFileExists(stringo),
            [regular_file_exists # elviWord(stringo)] => TestOptions::RegularFileExists(stringo),
            [file_exists_group_id # elviWord(stringo)] => TestOptions::FileExistsOwnerEffectiveGroupID(stringo),
            [symbolic_link # elviWord(stringo)] => TestOptions::SymbolicLinkExists(stringo),
            [sticky_bit_set # elviWord(stringo)] => TestOptions::StickyBitSetExists(stringo),
            [string_nonzero # elviWord(stringo)] => TestOptions::StringNonZero(stringo),
            [named_pipe # elviWord(stringo)] => TestOptions::NamedPipeExists(stringo),
            [readable_file # elviWord(stringo)] => TestOptions::ReadableFileExists(stringo),
            [greater_than_zero_file # elviWord(stringo)] => TestOptions::FileExistsGreaterThanZero(stringo),
            [file_descriptor # elviWord(stringo)] => TestOptions::FDDescriptorNumberOpened(stringo),
            [file_exists_user_id # elviWord(stringo)] => TestOptions::FileExistsUserIDSet(stringo),
            [writable_file # elviWord(stringo)] => TestOptions::FileExistsWritable(stringo),
            [efective_user_id_file # elviWord(stringo)] => TestOptions::FileExistsOwnerEffectiveUserID(stringo),
            [efective_group_id_file # elviWord(stringo)] => TestOptions::FileExistsOwnerEffectiveGroupID(stringo),
            [socket_file_exists # elviWord(stringo)] => TestOptions::FileExistsSocket(stringo),
        ))
    }

    pub fn builtinTestComparisons(input: Node) -> Result<TestOptions> {
        Ok(match_nodes!(input.into_children();
            [elviWord(stringo), string_equals # elviWord(stringo2)] => TestOptions::String1IsString2((stringo, stringo2)),
            [elviWord(stringo), string_not_equals # elviWord(stringo2)] => TestOptions::String1IsNotString2((stringo, stringo2)),
            [elviWord(stringo), ascii_comparison_lt # elviWord(stringo2)] => TestOptions::String1BeforeString2ASCII((stringo, stringo2)),
            [elviWord(stringo), ascii_comparison_gt # elviWord(stringo2)] => TestOptions::String1AfterString2ASCII((stringo, stringo2)),
            [elviWord(stringo), integer_eq # elviWord(stringo2)] => TestOptions::Int1EqualsInt2Algebraically((stringo, stringo2)),
            [elviWord(stringo), integer_ne # elviWord(stringo2)] => TestOptions::Int1NotEqualsInt2Algebraically((stringo, stringo2)),
            [elviWord(stringo), integer_gt # elviWord(stringo2)] => TestOptions::Int1GreaterThanInt2Algebraically((stringo, stringo2)),
            [elviWord(stringo), integer_ge # elviWord(stringo2)] => TestOptions::Int1GreaterEqualInt2Algebraically((stringo, stringo2)),
            [elviWord(stringo), integer_lt # elviWord(stringo2)] => TestOptions::Int1LessThanInt2Algebraically((stringo, stringo2)),
            [elviWord(stringo), integer_le # elviWord(stringo2)] => TestOptions::Int1LessEqualInt2Algebraically((stringo, stringo2)),
        ))
    }

    /// Handles the builtin `test`.
    pub fn builtinTest(input: Node) -> Result<Actions> {
        Ok(match_nodes!(input.into_children();
            [builtinTestComparisons(results)] => Actions::Builtin(Builtins::Test(results)),
            [builtinTestPrimaries(results)] => Actions::Builtin(Builtins::Test(results)),
        ))
    }

    pub fn elviSingleWord(input: Node) -> Result<ElviType> {
        Ok(ElviType::String(input.as_str().to_string()))
    }

    /// Handles any single word
    pub fn elviWord(input: Node) -> Result<ElviType> {
        Ok(match_nodes!(input.into_children();
            [anyString(stringo)] => stringo,
            [elviSingleWord(stringo)] => stringo,
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

    /// Handles the hash builtin.
    pub fn builtinHash(input: Node) -> Result<Actions> {
        let possibles = match_nodes!(input.into_children();
            [elviWord(stringo)] => Some(stringo),
            [] => None,
        );

        Ok(Actions::Builtin(Builtins::Hash(possibles)))
    }

    /// Handles the cd builtin.
    pub fn builtinCd(input: Node) -> Result<Actions> {
        let possibles = match_nodes!(input.into_children();
            [elviWord(stringo)] => Some(stringo),
            [] => None,
        );

        Ok(Actions::Builtin(Builtins::Cd(possibles)))
    }

    pub fn builtinWrapper(input: Node) -> Result<Actions> {
        Ok(match_nodes!(input.into_children();
            [builtinDbg(stringo)] => stringo,
            [builtinExit(stringo)] => stringo,
            [builtinUnset(stringo)] => stringo,
            [builtinHash(stringo)] => stringo,
            [builtinCd(stringo)] => stringo,
            [builtinTest(stringo)] => stringo,
        ))
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
            [builtinWrapper(var)] => Ok(var),
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
                        Actions::ChangeVariable((name, var)) => {
                            change_variable(&mut variables, &commands, subshells_in, name, &var);
                        }
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
                            Builtins::Hash(mut flag) => {
                                // Let's just eval possible vars
                                if flag.is_some() {
                                    flag = Some(flag.unwrap().eval_variables(&variables));
                                }
                                let ret =
                                    builtins::hash::builtin_hash(flag, &mut commands, &variables)
                                        .get();
                                variables.set_ret(ReturnCode::ret(ret));
                            }
                            Builtins::Cd(mut flag) => {
                                // Let's just eval possible vars
                                if flag.is_some() {
                                    flag = Some(flag.unwrap().eval_variables(&variables));
                                }
                                let ret = builtins::cd::builtin_cd(flag, &mut variables).get();
                                variables.set_ret(ReturnCode::ret(ret));
                            }
                            Builtins::Test(yo) => {
                                let ret = builtins::test::builtin_test(yo, &mut variables).get();
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

    #[test]
    fn single_quote_string_is_chill() {
        let stringo = r#"'foobar'"#;
        let parse = ElviParser::parse(Rule::singleQuoteString, stringo).unwrap();
        assert_eq!(r#"'foobar'"#, parse.as_str());
    }
}
