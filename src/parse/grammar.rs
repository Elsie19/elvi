use crate::internal::builtins;
use crate::internal::commands::Commands;
use crate::internal::status::ReturnCode;
use crate::internal::tree::{change_variable, Actions, Builtins, Conditional, Loop, TestOptions};
use crate::internal::variables::Arguments;
use crate::internal::variables::{ElviGlobal, ElviMutable, ElviType, Variable, Variables};
use pest_consume::{match_nodes, Error, Parser};

#[derive(Parser)]
#[grammar = "parse/internals/strings.pest"]
#[grammar = "parse/internals/variables.pest"]
#[grammar = "parse/internals/command_substitution.pest"]
#[grammar = "parse/internals/builtins.pest"]
#[grammar = "parse/internals/commands.pest"]
#[grammar = "parse/internals/if.pest"]
#[grammar = "parse/internals/for.pest"]
#[grammar = "parse/internals/base.pest"]
/// Global struct that implements the pest.rs parser ([`pest_derive`]).
pub struct ElviParser;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i, 'a> = pest_consume::Node<'i, Rule, &'a Arguments>;

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
            [backtickInner(backtick)] => backtick,
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
            [block # elviWord(path)] => TestOptions::BlockFileExists(path),
            [character_special # elviWord(path)] => TestOptions::CharacterFileExists(path),
            [directory_exists # elviWord(path)] => TestOptions::DirectoryExists(path),
            [file_exists # elviWord(path)] => TestOptions::AnyFileExists(path),
            [regular_file_exists # elviWord(path)] => TestOptions::RegularFileExists(path),
            [file_exists_group_id # elviWord(path)] => TestOptions::FileExistsOwnerEffectiveGroupID(path),
            [symbolic_link # elviWord(path)] => TestOptions::SymbolicLinkExists(path),
            [sticky_bit_set # elviWord(path)] => TestOptions::StickyBitSetExists(path),
            [string_nonzero # elviWord(path)] => TestOptions::StringNonZero(path),
            [string_zero # elviWord(path)] => TestOptions::StringZero(path),
            [named_pipe # elviWord(path)] => TestOptions::NamedPipeExists(path),
            [readable_file # elviWord(path)] => TestOptions::ReadableFileExists(path),
            [greater_than_zero_file # elviWord(path)] => TestOptions::FileExistsGreaterThanZero(path),
            [file_descriptor # elviWord(path)] => TestOptions::FDDescriptorNumberOpened(path),
            [file_exists_user_id # elviWord(path)] => TestOptions::FileExistsUserIDSet(path),
            [writable_file # elviWord(path)] => TestOptions::FileExistsWritable(path),
            [efective_user_id_file # elviWord(path)] => TestOptions::FileExistsOwnerEffectiveUserID(path),
            [efective_group_id_file # elviWord(path)] => TestOptions::FileExistsOwnerEffectiveGroupID(path),
            [socket_file_exists # elviWord(path)] => TestOptions::FileExistsSocket(path),
        ))
    }

    pub fn builtinTestComparisons(input: Node) -> Result<TestOptions> {
        Ok(match_nodes!(input.into_children();
            [elviWord(stringo), string_equals # elviWord(stringo2)] => TestOptions::String1IsString2((stringo, stringo2)),
            [elviWord(stringo), string_not_equals # elviWord(stringo2)] => TestOptions::String1IsNotString2((stringo, stringo2)),
            [elviWord(stringo), ascii_comparison_lt # elviWord(stringo2)] => TestOptions::String1BeforeString2ASCII((stringo, stringo2)),
            [elviWord(stringo), ascii_comparison_gt # elviWord(stringo2)] => TestOptions::String1AfterString2ASCII((stringo, stringo2)),
            [elviWord(n1), integer_eq # elviWord(n2)] => TestOptions::Int1EqualsInt2Algebraically((n1, n2)),
            [elviWord(n1), integer_ne # elviWord(n2)] => TestOptions::Int1NotEqualsInt2Algebraically((n1, n2)),
            [elviWord(n1), integer_gt # elviWord(n2)] => TestOptions::Int1GreaterThanInt2Algebraically((n1, n2)),
            [elviWord(n1), integer_ge # elviWord(n2)] => TestOptions::Int1GreaterEqualInt2Algebraically((n1, n2)),
            [elviWord(n1), integer_lt # elviWord(n2)] => TestOptions::Int1LessThanInt2Algebraically((n1, n2)),
            [elviWord(n1), integer_le # elviWord(n2)] => TestOptions::Int1LessEqualInt2Algebraically((n1, n2)),
        ))
    }

    pub fn builtinTestInvert(_input: Node) -> Result<bool> {
        Ok(true)
    }

    /// Handles the builtin `test`.
    pub fn builtinTest(input: Node) -> Result<Actions> {
        Ok(match_nodes!(input.into_children();
            [builtinTestComparisons(results)] | [builtinTestPrimaries(results)] => Actions::Builtin(Builtins::Test(false, results)),
            [builtinTestInvert(_char), builtinTestComparisons(results)] | [builtinTestInvert(_char), builtinTestPrimaries(results)] => Actions::Builtin(Builtins::Test(true, results)),
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
                &variable_contents.unwrap(),
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
                &variable_contents.unwrap(),
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

    /// Handles the echo builtin.
    pub fn builtinEcho(input: Node) -> Result<Actions> {
        let possibles = match_nodes!(input.into_children();
            [elviWord(stringo)..] => Some(stringo.collect()),
            [] => None,
        );

        Ok(Actions::Builtin(Builtins::Echo(possibles)))
    }

    /// Handles the exit builtin.
    pub fn builtinExit(input: Node) -> Result<Actions> {
        let possibles = match_nodes!(input.into_children();
            [elviWord(stringo)] => Some(stringo),
            [elviNumber(num)] => Some(ElviType::String(num.to_string())),
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
            [builtinEcho(stringo)] => stringo,
        ))
    }

    /// Handles any external command.
    pub fn externalCommand(input: Node) -> Result<Actions> {
        Ok(Actions::Command(vec!["dbgbar".to_string()]))
    }

    /// Handles if statement conditions
    pub fn ifStatementMatch(input: Node) -> Result<Actions> {
        Ok(match_nodes!(input.into_children();
            [builtinWrapper(built)] => built,
        ))
    }

    /// Handles if statements.
    pub fn ifStatement(input: Node) -> Result<Actions> {
        Ok(match_nodes!(input.into_children();
            // Condition + then_block
            [ifStatementMatch(condition), then_block # statement(stmt)..] => Actions::IfStatement(Box::new(
            Conditional {
                condition,
                then_block: stmt.collect(),
                elif_block: None,
                else_block: None
            }
        ),
        ),
            // Condition + then_block + else_block
            [ifStatementMatch(condition), then_block # statement(stmt).., else_block # statement(else_part)..] => Actions::IfStatement(Box::new(
                        Conditional {
                            condition,
                            then_block: stmt.collect(),
                            elif_block: None,
                            else_block: Some(else_part.collect())
                        }
                    ),
                    )
                ))
    }

    /// Handles the inner matching of for loops
    pub fn forLoopMatch(input: Node) -> Result<ElviType> {
        Ok(match_nodes!(input.into_children();
            [backtickSubstitution(tick)] => tick,
            [elviWord(word)] => word,
        ))
    }

    /// Handles for loops.
    pub fn forLoop(input: Node) -> Result<Actions> {
        Ok(match_nodes!(input.into_children();
            // When we have loop contents
            [variable # elviWord(var), loop_match # forLoopMatch(loop_match).., inner_for # statement(stmt)..] => Actions::ForLoop(Loop { variable: var, elements: loop_match.collect(), do_block: stmt.collect() }),
            // When we don't
            [variable # elviWord(var), inner_for # statement(stmt)..] => Actions::ForLoop(Loop { variable: var, elements: vec![ElviType::VariableSubstitution("${@}".to_string())], do_block: stmt.collect() })
        ))
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
            [ifStatement(stmt)] => Ok(stmt),
            [forLoop(stmt)] => Ok(stmt),
        )
    }

    /// Entry point for parsing.
    pub fn program(input: Node) -> ReturnCode {
        let mut variables = Variables::default();
        let mut commands = Commands::generate(&variables);

        let positional_arguments = input.user_data();

        // Set all the positional variables once.
        for (idx, elem) in positional_arguments.args.iter().enumerate() {
            variables
                .set_variable(
                    idx.to_string(),
                    Variable::oneshot_var(
                        &ElviType::String(elem.to_string()),
                        ElviMutable::Normal,
                        ElviGlobal::Global,
                        (0, 0),
                    ),
                )
                .unwrap();
        }

        let mut subshells_in = 1;

        for child in input.into_children() {
            if child.as_rule() != Rule::EOI {
                match Self::statement(child) {
                    Ok(yes) => {
                        eval(yes, &mut variables, &mut commands, &mut subshells_in);
                    }
                    Err(oops) => {
                        eprintln!("{oops}");
                        continue;
                    }
                }
            }
        }

        ReturnCode::ret(variables.get_ret().convert_err_type().get())
    }
}

/// Evaluates any given [`Actions`].
pub fn eval(
    action: Actions,
    variables: &mut Variables,
    commands: &mut Commands,
    subshells_in: &mut u32,
) -> ReturnCode {
    match action {
        Actions::ChangeVariable((name, mut var)) => {
            change_variable(variables, commands, *subshells_in, name, &mut var);
        }
        Actions::Builtin(built) => match built {
            Builtins::Dbg(var) => {
                let ret = builtins::dbg::builtin_dbg(&var, variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
            Builtins::Exit(var) => {
                let ret = builtins::exit::builtin_exit(var);
                if *subshells_in > 1 {
                    *subshells_in -= 1;
                } else {
                    std::process::exit(ret.get().into());
                }
            }
            Builtins::Unset(var) => {
                let ret = builtins::unset::builtin_unset(&var, variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
            Builtins::Hash(mut flag) => {
                // Let's just eval possible vars
                if flag.is_some() {
                    flag = Some(flag.unwrap().eval_escapes().eval_variables(variables));
                }
                let ret = builtins::hash::builtin_hash(flag, commands, variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
            Builtins::Cd(mut flag) => {
                // Let's just eval possible vars
                if flag.is_some() {
                    flag = Some(flag.unwrap().eval_escapes().eval_variables(variables));
                }
                let ret = builtins::cd::builtin_cd(flag, variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
            Builtins::Test(invert, yo) => {
                let ret = builtins::test::builtin_test(invert, yo, variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
            Builtins::Echo(text) => {
                let ret = builtins::echo::builtin_echo(text, variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
        },
        Actions::Command(cmd) => {
            println!("Running command {cmd:?}");
        }
        Actions::Null => {}
        Actions::IfStatement(if_stmt) => {
            // Run the condition
            eval(if_stmt.condition, variables, commands, subshells_in);
            // Did we succeed?
            if variables.get_ret().convert_err_type().get() == ReturnCode::SUCCESS {
                for act in if_stmt.then_block {
                    let ret = eval(act, variables, commands, subshells_in);
                    variables.set_ret(ret);
                }
            } else if if_stmt.else_block.is_some() {
                for act in if_stmt.else_block.unwrap() {
                    let ret = eval(act, variables, commands, subshells_in);
                    variables.set_ret(ret);
                }
            }
        }
        Actions::ForLoop(loop_things) => {
            let mut new_loop_elements = vec![];
            for element in &loop_things.elements {
                for entry in element
                    .tilde_expansion(variables)
                    .eval_variables(variables)
                    .expand_globs()
                {
                    new_loop_elements.push(entry);
                }
            }
            for var in &new_loop_elements {
                // Ok so now I want to update the variable if it exists before, and if not, create a
                // new variable.
                if variables
                    .get_variable(&loop_things.variable.to_string())
                    .is_some()
                {
                    let template = variables
                        .get_variable(&loop_things.variable.to_string())
                        .unwrap();
                    match variables.set_variable(
                        loop_things.variable.to_string(),
                        Variable::oneshot_var(
                            var,
                            template.get_modification_status(),
                            template.get_lvl(),
                            template.get_line(),
                        ),
                    ) {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("{e}");
                            std::process::exit(ReturnCode::FAILURE.into());
                        }
                    }
                } else {
                    variables.set_variable(
                        loop_things.variable.to_string(),
                        Variable::oneshot_var(
                            var,
                            ElviMutable::Normal,
                            ElviGlobal::Global,
                            (0, 0),
                        ),
                    ).unwrap() /* I'm reasonably confident that this won't fail */;
                }
                for act in &loop_things.do_block {
                    let ret = eval(act.to_owned(), variables, commands, subshells_in);
                    variables.set_ret(ret);
                }
            }
        }
    }
    ReturnCode::ret(variables.get_ret().convert_err_type().get())
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
