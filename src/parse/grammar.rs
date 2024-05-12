use crate::internal::variables::{ElviGlobal, ElviMutable, ElviType, Variable};
use pest_consume::{match_nodes, Error, Parser};

#[derive(Parser)]
#[grammar = "parse/internals/strings.pest"]
#[grammar = "parse/internals/variables.pest"]
#[grammar = "parse/internals/command_substitution.pest"]
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

        let name_pair = stuff.next().unwrap().as_str();

        let variable_contents =
            Self::variableIdentifierPossibilities(input.into_children().nth(1).unwrap());

        Ok((
            name_pair.to_string(),
            Variable::oneshot_var(
                variable_contents.unwrap(),
                ElviMutable::Normal,
                ElviGlobal::Normal(1),
            ),
        ))
    }

    pub fn readonlyVariable(input: Node) -> Result<(String, Variable)> {
        let mut stuff = input.clone().into_children().into_pairs();

        let name_pair = stuff.next().unwrap().as_str();

        let variable_contents =
            Self::variableIdentifierPossibilities(input.into_children().nth(1).unwrap());

        Ok((
            name_pair.to_string(),
            Variable::oneshot_var(
                variable_contents.unwrap(),
                ElviMutable::Readonly,
                ElviGlobal::Normal(1),
            ),
        ))
    }

    pub fn program(input: Node) -> Result<Vec<(String, Variable)>> {
        let mut le_vars: Vec<(String, Variable)> = vec![];
        match_nodes!(input.into_children();
            [normalVariable(var).., _] => {
                let le_vars_collected: Vec<(String, Variable)> = var.collect();
                for indi_var in le_vars_collected {
                    le_vars.push(indi_var);
                }
            },
            [readonlyVariable(var).., _] => {
                let le_vars_collected: Vec<(String, Variable)> = var.collect();
                for indi_var in le_vars_collected {
                    le_vars.push(indi_var);
                }
            },
        );
        Ok(le_vars)
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
