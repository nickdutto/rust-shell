use crate::parser::span::Span;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Error)]
pub enum ParserErrorKind {
    #[error(
        "Invalid variable name. Must start with ASCII alphabetic or _ and contain only ASCII alphanumeric or _"
    )]
    InvalidVariableName,
    #[error("Variable is missing a closing brace")]
    UnclosedVariableBrace,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub span: Span,
    pub raw_string: String,
}
