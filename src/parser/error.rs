use crate::parser::span::Span;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Error)]
pub enum ParserErrorKind {}

#[derive(Clone, Debug, PartialEq)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub span: Span,
    pub raw_string: String,
}
