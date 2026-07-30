use crate::parser::error::{ParserError, ParserErrors};
use crate::parser::span::Span;
use miette::Diagnostic;
use std::io;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ShellError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Parser(#[from] ParserError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ParserMulti(#[from] ParserErrors),

    #[error(transparent)]
    Io(#[from] io::Error),
}
