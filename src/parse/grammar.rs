use crate::internal::commands::Commands;
use crate::internal::tree::Actions;
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

    pub fn normalVariable(input: Node) -> Result<()> {
        let mut stuff = input.clone().into_children().into_pairs();

        let lines = stuff.clone().next().unwrap().line_col();

        let name_pair = stuff.next().unwrap().as_str();

        let variable_contents =
            Self::variableIdentifierPossibilities(input.clone().into_children().nth(1).unwrap());

        Ok(())

        // match input.user_data().0.set_variable(
        //     name_pair.to_string(),
        //     Variable::oneshot_var(
        //         variable_contents.unwrap(),
        //         ElviMutable::Normal,
        //         ElviGlobal::Normal(1),
        //         lines,
        //     ),
        // ) {
        //     Ok(_) => Ok(()),
        //     Err(foo) => {
        //         eprintln!("{foo}");
        //         Ok(())
        //     }
        // }
    }

    pub fn readonlyVariable(input: Node) -> Result<()> {
        let mut stuff = input.clone().into_children().into_pairs();

        let lines = stuff.clone().next().unwrap().line_col();

        let name_pair = stuff.next().unwrap().as_str();

        let variable_contents =
            Self::variableIdentifierPossibilities(input.clone().into_children().nth(1).unwrap());

        Ok(())

        // match input.user_data().0.set_variable(
        //     name_pair.to_string(),
        //     Variable::oneshot_var(
        //         variable_contents.unwrap(),
        //         ElviMutable::Readonly,
        //         ElviGlobal::Normal(1),
        //         lines,
        //     ),
        // ) {
        //     Ok(_) => Ok(()),
        //     Err(foo) => {
        //         eprintln!("{foo}");
        //         Ok(())
        //     }
        // }
    }

    pub fn builtinDbg(input: Node) -> Result<()> {
        let name = input
            .into_children()
            .into_pairs()
            .next()
            .unwrap()
            .as_str()
            .to_string();

        // println!(
        //     "Variable: {} | Contents: {:?}",
        //     name,
        //     input.user_data().0.get_variable(&name)
        // );
        Ok(())
    }

    pub fn externalCommand(input: Node) -> Result<Actions> {
        Ok(Actions::Command(vec!["dbgbar".to_string()]))
    }

    pub fn ifStatement(input: Node) -> Result<()> {
        Ok(())
    }

    pub fn statement(input: Node) -> Result<()> {
        match_nodes!(input.into_children();
            [normalVariable(var)] | [readonlyVariable(var)] => {
                Ok(())
            },
            // [builtinDbg(var)] => Ok(var),
            // [externalCommand(var)] => Ok(var),
            // [ifStatement(var)] => Ok(var),
        )
    }

    pub fn program(input: Node) -> Result<()> {
        let mut variables = Variables::default();
        let mut commands = Commands::generate(&variables);

        let mut nodes = input.into_children();
        dbg!(nodes);
        Ok(())
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
