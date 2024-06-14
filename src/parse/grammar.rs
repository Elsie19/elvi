use crate::internal::builtins;
use crate::internal::commands::{execute_external_command, Commands, ExternalCommand};
use crate::internal::errors::ElviError;
use crate::internal::status::ReturnCode;
use crate::internal::tree::Function;
use crate::internal::tree::{change_variable, Actions, Builtins, Conditional, Loop, TestOptions};
use crate::internal::variables::Arguments;
use crate::internal::variables::{ElviGlobal, ElviMutable, ElviType, Variable, Variables};
use pest_consume::{match_nodes, Error, Parser};

#[derive(Parser)]
#[grammar = "parse/internals/base.pest"]
#[grammar = "parse/internals/strings.pest"]
#[grammar = "parse/internals/variables.pest"]
#[grammar = "parse/internals/command_substitution.pest"]
#[grammar = "parse/internals/builtins.pest"]
#[grammar = "parse/internals/commands.pest"]
#[grammar = "parse/internals/if.pest"]
#[grammar = "parse/internals/for.pest"]
#[grammar = "parse/internals/functions.pest"]
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

    pub fn builtinTestInvert(input: Node) -> Result<bool> {
        Ok(true)
    }

    /// Handles the builtin `test`.
    pub fn builtinTest(input: Node) -> Result<Actions> {
        Ok(match_nodes!(input.into_children();
            [builtinTestComparisons(results)] | [builtinTestPrimaries(results)] => Actions::Builtin(Builtins::Test(false, results)),
            [invert # builtinTestInvert(_char), builtinTestComparisons(results)] | [invert # builtinTestInvert(_char), builtinTestPrimaries(results)] => Actions::Builtin(Builtins::Test(true, results)),
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
            Variable {
                contents: variable_contents.unwrap(),
                shell_lvl: ElviGlobal::Normal(1),
                line: lines,
                ..Default::default()
            },
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
            Variable {
                contents: variable_contents.unwrap(),
                shell_lvl: ElviGlobal::Normal(1),
                modification_status: ElviMutable::Readonly,
                line: lines,
            },
        ))
    }

    /// Handles the readonly builtin.
    pub fn builtinDbg(input: Node) -> Result<Actions> {
        let possibles = match_nodes!(input.into_children();
            [elviWord(stringo)..] => Some(stringo.collect()),
            [] => None,
        );

        Ok(Actions::Builtin(Builtins::Dbg(possibles)))
    }

    /// Handles the unset builtin.
    pub fn builtinUnset(input: Node) -> Result<Actions> {
        let possibles = match_nodes!(input.into_children();
            [elviWord(stringo)..] => Some(stringo.collect()),
            [] => None,
        );

        Ok(Actions::Builtin(Builtins::Unset(possibles)))
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
            [elviWord(stringo)..] => Some(stringo.collect()),
            [] => None,
        );

        Ok(Actions::Builtin(Builtins::Exit(possibles)))
    }

    /// Handles the hash builtin.
    pub fn builtinHash(input: Node) -> Result<Actions> {
        let possibles = match_nodes!(input.into_children();
            [elviWord(stringo)..] => Some(stringo.collect()),
            [] => None,
        );

        Ok(Actions::Builtin(Builtins::Hash(possibles)))
    }

    /// Handles the cd builtin.
    pub fn builtinCd(input: Node) -> Result<Actions> {
        let possibles = match_nodes!(input.into_children();
            [elviWord(stringo)..] => Some(stringo.collect()),
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
        Ok(match_nodes!(input.into_children();
            [elviWord(yep)..] => Actions::Command(yep.collect()),
        ))
    }

    /// Handles function statements.
    pub fn functionDeclaration(input: Node) -> Result<Actions> {
        Ok(match_nodes!(input.into_children();
            [name # variableIdent(name), inner_function # statement(stmt)..] => {
                Actions::FunctionDeclaration(Function {
                    name,
                    contents: Some(stmt.collect()),
                })
            },
            [name # variableIdent(name)] => {
                Actions::FunctionDeclaration(Function {
                    name,
                    // This is to match when I run `foo() {}` then `foo` in sh.
                    // TODO: Change this to a compound command later.
                    contents: Some(vec![Actions::Command(vec![ElviType::String("{}".into())])]),
                })
            }
        ))
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
            [normalVariable(var)] => Ok(Actions::ChangeVariable(var)),
            [readonlyVariable(var)] => Ok(Actions::ChangeVariable(var)),
            [builtinWrapper(var)] => Ok(var),
            [externalCommand(var)] => Ok(var),
            [ifStatement(stmt)] => Ok(stmt),
            [forLoop(stmt)] => Ok(stmt),
            [functionDeclaration(func)] => Ok(func),
        )
    }

    /// Entry point for parsing.
    pub fn program(input: Node) -> ReturnCode {
        let mut variables = Variables::default();
        let mut commands = Commands::generate(&variables);

        let positional_arguments = input.user_data();

        // Set all the positional variables once.
        let list: Vec<Variable> = positional_arguments
            .args
            .clone()
            .iter()
            .map(|var| var.to_owned().into())
            .collect();
        variables.new_parameters(&list);

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
// We know clippy. Lol.
#[allow(clippy::too_many_lines)]
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
                let ret = builtins::dbg::builtin_dbg(var.as_deref(), variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
            Builtins::Exit(var) => {
                let ret = builtins::exit::builtin_exit(var.as_deref(), variables);
                if *subshells_in > 1 {
                    *subshells_in -= 1;
                } else {
                    std::process::exit(ret.get().into());
                }
            }
            Builtins::Unset(var) => {
                let ret = builtins::unset::builtin_unset(var.as_deref(), variables, commands).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
            Builtins::Hash(flag) => {
                // Let's just eval possible vars
                let ret = builtins::hash::builtin_hash(flag.as_deref(), commands, variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
            Builtins::Cd(flag) => {
                // Let's just eval possible vars
                let ret = builtins::cd::builtin_cd(flag.as_deref(), variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
            Builtins::Test(invert, yo) => {
                let ret = builtins::test::builtin_test(invert, yo, variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
            Builtins::Echo(text) => {
                let ret = builtins::echo::builtin_echo(text.as_deref(), variables).get();
                variables.set_ret(ReturnCode::ret(ret));
            }
        },
        Actions::Command(cmd) => {
            let mut expanded = vec![];
            for part in cmd {
                expanded.push(
                    part.tilde_expansion(variables)
                        .eval_escapes()
                        .eval_variables(variables)
                        .to_string(),
                );
            }
            let cmd_run: ExternalCommand = expanded.into();
            let templated = execute_external_command(cmd_run, variables, commands);
            match templated {
                Ok(mut yay) => {
                    let mut foop = match yay.spawn() {
                        Ok(yes) => yes,
                        Err(f) => {
                            eprintln!("{f}");
                            variables.set_ret(ReturnCode::FAILURE.into());
                            return variables.get_ret().convert_err_type();
                        }
                    };
                    variables.set_ret(foop.wait().unwrap().code().unwrap().into());
                }
                Err(oops) => {
                    eprintln!("{oops}");
                    variables.set_ret(oops.ret());
                }
            }
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
                        Variable {
                            contents: var.clone(),
                            modification_status: template.modification_status,
                            shell_lvl: template.shell_lvl,
                            line: template.line,
                        },
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
                        Variable { contents: var.clone(), ..Default::default() }
                    ).unwrap() /* I'm reasonably confident that this won't fail */;
                }
                for act in &loop_things.do_block {
                    let ret = eval(act.to_owned(), variables, commands, subshells_in);
                    variables.set_ret(ret);
                }
            }
        }
        Actions::FunctionDeclaration(func) => {
            commands.register_function(func);
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
