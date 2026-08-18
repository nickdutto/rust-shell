use crate::parser::lexer::{Token, TokenKind};
use crate::parser::span::Span;
use crate::parser::word::Word;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Clone, Debug, Error, Diagnostic, PartialEq)]
pub enum ParserError {
    #[error(
        "Invalid variable name. Must start with ASCII alphabetic or _ and contain only ASCII alphanumeric or _"
    )]
    #[diagnostic(
        code(rs_shell::parser::invalid_variable_name),
        help("Variable names must match [a-zA-Z_][a-zA-Z0-9_]*")
    )]
    InvalidVariableName {
        #[label("invalid variable name here")]
        span: Span,
    },

    #[error("Variable is missing a closing brace")]
    #[diagnostic(
        code(rs_shell::parser::unclosed_brace),
        help("Add a closing '}}' to balance '${{'")
    )]
    UnclosedVariableBrace {
        #[label("missing '}}'")]
        span: Span,
    },
}

impl ParserError {
    pub fn span(&self) -> Span {
        match self {
            ParserError::InvalidVariableName { span }
            | ParserError::UnclosedVariableBrace { span } => *span,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Found {count} parser errors")]
#[diagnostic(code(rs_shell::parser_multi_error))]
pub struct ParserMultiError {
    pub count: usize,
    #[related]
    pub errors: Vec<ParserError>,
}

pub fn collect_parser_errors(tokens: &[Token]) -> ParserMultiError {
    let mut errors = vec![];

    for token in tokens {
        if let TokenKind::Word(words) = &token.kind {
            for word in words {
                if let Word::Error(_, err) = &word.item {
                    errors.push(err.clone());
                }
            }
        }
    }

    ParserMultiError {
        count: errors.len(),
        errors,
    }
}
