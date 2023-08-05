use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"] // relative to src
pub struct EndpointParser;

type TypeName = String;

impl EndpointParser {
    pub fn parse_endpoint(input: &str) -> Result<Endpoint, ParseError> {
        let value = Self::parse(Rule::endpoint, input)
            .map_err(Box::new)?
            .next()
            .unwrap();

        match value.as_rule() {
            Rule::endpoint => value.try_into(),
            _ => panic!("unreachable"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Unexpect rule")]
    UnexpectRule,
    #[error("Unsupport type")]
    UnsupportType,
    #[error("Parse error")]
    PestError(#[from] Box<pest::error::Error<Rule>>),
}

impl TryFrom<Pair<'_, Rule>> for Endpoint {
    type Error = ParseError;

    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        if Rule::endpoint == value.as_rule() {
            let mut inner = value.into_inner();
            let method: Method = inner.next().unwrap().try_into()?;
            let path: Vec<Path> = inner
                .next()
                .unwrap()
                .into_inner()
                .map(|v| v.try_into())
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let query_params: Vec<Variable> = inner
                .next()
                .unwrap()
                .into_inner()
                .map(|v| v.try_into())
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let mut pair = inner.next();
            let req_type: Option<RequestType> = if let Some(Ok(rq)) = pair.as_ref().map(|v| v.try_into()) {
                pair = inner.next();
                Some(rq)
            } else {
                None
            };
            let res_type: Option<ResponseType> = pair.and_then(|v| v.try_into().ok());

            Ok(Self {
                method,
                path,
                query_params,
                request_type: req_type.map(|v| v.0),
                response_type: res_type.map(|v| v.0),
            })
        } else {
            Err(ParseError::UnexpectRule)
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Endpoint {
    pub method: Method,
    pub path: Vec<Path>,
    pub query_params: Vec<Variable>,
    pub request_type: Option<TypeName>,
    pub response_type: Option<TypeName>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Path {
    Segment(String),
    Variable(String, VariableType),
}

#[derive(Debug)]
pub struct QueryParam(String, VariableType);

#[derive(Debug, PartialEq, PartialOrd)]
pub enum VariableType {
    String,
    Short,
    Int,
    Long,
    Float,
    Double,
    Bool,
}

#[derive(Debug)]
pub struct RequestType(String);
#[derive(Debug)]
pub struct ResponseType(String);

impl TryFrom<Pair<'_, Rule>> for VariableType {
    type Error = ParseError;

    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        if Rule::variable_type == value.as_rule() {
            match value.as_str().to_lowercase().as_str() {
                "string" => Ok(Self::String),
                "short" => Ok(Self::Short),
                "int" => Ok(Self::Int),
                "long" => Ok(Self::Long),
                "float" => Ok(Self::Float),
                "double" => Ok(Self::Double),
                "bool" => Ok(Self::Bool),
                _ => Err(ParseError::UnsupportType),
            }
        } else {
            Err(ParseError::UnexpectRule)
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Variable(String, VariableType);

impl TryFrom<Pair<'_, Rule>> for Variable {
    type Error = ParseError;

    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        if Rule::variable == value.as_rule() {
            let mut pairs = value.into_inner();
            let name = pairs.next().unwrap();
            let variable_type = pairs.next().unwrap().try_into()?;

            Ok(Self(name.as_str().to_owned(), variable_type))
        } else {
            Err(ParseError::UnexpectRule)
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for Method {
    type Error = ParseError;

    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        if Rule::method == value.as_rule() {
            let Some(method) = value.into_inner().next() else {panic!("Impossible")};
            Ok(match method.as_str().to_uppercase().as_str() {
                "GET" => Self::GET,
                "POST" => Self::POST,
                "PUT" => Self::PUT,
                "DELETE" => Self::DELETE,
                _ => panic!("Impossible"),
            })
        } else {
            Err(ParseError::UnexpectRule)
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for Path {
    type Error = ParseError;

    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        if Rule::segment == value.as_rule() {
            Ok(Path::Segment(
                value.into_inner().next().unwrap().as_str().to_string(),
            ))
        } else if Rule::variable == value.as_rule() {
            let Variable(name, var_type) = value.try_into()?;
            Ok(Path::Variable(name, var_type))
        } else {
            Err(ParseError::UnexpectRule)
        }
    }
}

macro_rules! impl_string_type {
    ($type:ident, $rule:expr, $r:ty) => {
        impl TryFrom<$r> for $type {
            type Error = ParseError;

            fn try_from(value: $r) -> Result<Self, Self::Error> {
                if $rule == value.as_rule() {
                    Ok($type(value.as_str().to_string()))
                } else {
                    Err(ParseError::UnexpectRule)
                }
            }
        }
    };
}

impl_string_type!(ResponseType, Rule::response_type, Pair<'_, Rule>);
impl_string_type!(RequestType, Rule::request_type, Pair<'_, Rule>);

impl_string_type!(ResponseType, Rule::response_type, &Pair<'_, Rule>);
impl_string_type!(RequestType, Rule::request_type, &Pair<'_, Rule>);

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::*;
    use crate::EndpointParser;

    #[test]
    fn test_variable() -> anyhow::Result<()> {
        let mut pairs = EndpointParser::parse(Rule::variable, "Name:string")?;
        let _: Variable = pairs.next().unwrap().try_into()?;
        Ok(())
    }

    #[test]
    fn test_method() -> anyhow::Result<()> {
        let mut pairs = EndpointParser::parse(Rule::method, "GET")?;
        let var: Method = pairs.next().unwrap().try_into()?;
        assert!(var == Method::GET);
        Ok(())
    }

    #[test]
    fn test_path() -> anyhow::Result<()> {
        let mut pairs = EndpointParser::parse(Rule::path, "/seg/{var:string}")?;

        for ele in pairs.next().unwrap().into_inner() {
            let _path: Path = ele.try_into()?;
        }
        Ok(())
    }

    #[test]
    fn test_query_params() -> anyhow::Result<()> {
        let mut pairs = EndpointParser::parse(Rule::query_params, "?a:string&b:bool")?;
        let inner = pairs.next().unwrap().into_inner();

        let _params: Result<Vec<Variable>, _> = inner.into_iter().map(|v| v.try_into()).collect();
        Ok(())
    }

    #[test]
    fn test_sig() -> anyhow::Result<()> {
        let mut pairs = EndpointParser::parse(Rule::request_type, "fasd")?;
        let request_type: RequestType = pairs.next().unwrap().try_into()?;

        println!("{:?}", request_type);

        Ok(())
    }

    #[test]
    fn test_endpoint() -> anyhow::Result<()> {
        let endpoint = EndpointParser::parse_endpoint(
            "GET /register/{id:string}?type:string&order:string RQ -> RS",
        )?;
        assert_eq!(
            Endpoint {
                method: Method::GET,
                path: vec![
                    Path::Segment("register".to_owned()),
                    Path::Variable("id".to_owned(), VariableType::String)
                ],
                query_params: vec![
                    Variable("type".to_owned(), VariableType::String),
                    Variable("order".to_owned(), VariableType::String),
                ],
                request_type: Some("RQ".to_owned()),
                response_type: Some("RS".to_owned())
            },
            endpoint
        );
        Ok(())
    }

    #[test]
    fn test_endpoint_without_sig() -> anyhow::Result<()> {
        let endpoint =
            EndpointParser::parse_endpoint("GET /register/{id:string}?type:string&order:string ")?;
        assert_eq!(
            Endpoint {
                method: Method::GET,
                path: vec![
                    Path::Segment("register".to_owned()),
                    Path::Variable("id".to_owned(), VariableType::String)
                ],
                query_params: vec![
                    Variable("type".to_owned(), VariableType::String),
                    Variable("order".to_owned(), VariableType::String),
                ],
                request_type: None,
                response_type: None
            },
            endpoint
        );
        Ok(())
    }
    #[test]
    fn test_endpoint_without_query_params() -> anyhow::Result<()> {
        let endpoint =
            EndpointParser::parse_endpoint("GET /register/{id:string} RQ -> RS")?;
        assert_eq!(
            Endpoint {
                method: Method::GET,
                path: vec![
                    Path::Segment("register".to_owned()),
                    Path::Variable("id".to_owned(), VariableType::String)
                ],
                query_params: vec![
                ],
                request_type: Some("RQ".to_owned()),
                response_type: Some("RS".to_owned())
            },
            endpoint
        );
        Ok(())
    }
}
