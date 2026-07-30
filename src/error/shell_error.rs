use crate::parser::error::{ParserError, ParserMultiError};
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
    ParserMulti(#[from] ParserMultiError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("External command failed")]
    #[diagnostic(code(rs_shell::external_command), help("{help}"))]
    ExternalCommand {
        help: String,
        label: String,
        #[label("{label}")]
        span: Span,
    },
}
