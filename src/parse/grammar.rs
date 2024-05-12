use crate::internal::variables::{ElviGlobal, ElviMutable, ElviType, Variable};
use pest_consume::{match_nodes, Error, Parser};
use snailquote::unescape;

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

        let foo =
            Self::variableIdentifierPossibilities(input.into_children().skip(1).next().unwrap());

        Ok((
            name_pair.to_string(),
            Variable::oneshot_var(foo.unwrap(), ElviMutable::Normal, ElviGlobal::Normal(1)),
        ))
    }

    pub fn program(input: Node) -> Result<Vec<(String, Variable)>> {
        Ok(match_nodes!(input.into_children();
            [normalVariable(var).., _] => {
                var.collect()
            },
        ))
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
