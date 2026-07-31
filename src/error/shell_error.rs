use crate::parser::error::{ParserError, ParserMultiError};
use crate::parser::span::Span;
use miette::Diagnostic;
use std::io;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ShellError {
    #[error("Multiple shell errors")]
    Multiple(Vec<ShellError>),

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

    #[error("Command argument error")]
    #[diagnostic(code(rs_shell::command_argument), help("{help}"))]
    CommandArgument {
        help: String,
        label: String,
        #[label("{label}")]
        span: Span,
    },
}
