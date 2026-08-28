use crate::parser::error::{ParserError, ParserMultiError};
use crate::value::span::Span;
use miette::{Diagnostic, NamedSource, Report};
use std::io;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ShellError {
    #[error("{0}")]
    Generic(String),

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

    #[error(transparent)]
    Ureq(#[from] ureq::Error),

    #[error("Command interrupted")]
    #[diagnostic(code(rs_shell::interrupted))]
    Interrupted {
        #[label("This command was interrupted")]
        span: Span,
    },

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

    #[error("Type mismatch: expected `{expected}`, got `{actual}`")]
    #[diagnostic(code(rs_shell::type_mismatch), help("Provide type `{expected}`"))]
    TypeMismatch {
        expected: String,
        actual: String,
        #[label("expected `{expected}`, got `{actual}`")]
        span: Span,
    },

    #[error("Unknown named argument `{name}` for command `{cmd}`")]
    #[diagnostic(code(rs_shell::unknown_named_argument))]
    UnknownNamedArgument {
        cmd: String,
        name: String,
        #[label("unknown named argument")]
        span: Span,
    },

    #[error("Too many arguments provided to `{cmd}`")]
    #[diagnostic(code(rs_shell::too_many_arguments))]
    TooManyArguments {
        cmd: String,
        #[label("unexpected extra argument")]
        span: Span,
    },

    #[error("Missing required positional argument `{name}` for command `{cmd}`")]
    #[diagnostic(code(rs_shell::missing_positional_argument))]
    MissingPositionalArgument {
        cmd: String,
        name: String,
        #[label("missing positional argument `{name}`")]
        span: Span,
    },

    #[error("Missing required named argument `{name}` for command `{cmd}`")]
    #[diagnostic(code(rs_shell::missing_named_argument))]
    MissingNamedArgument {
        cmd: String,
        name: String,
        #[label("missing named argument `{name}`")]
        span: Span,
    },

    #[error("Missing value for named argument `{name}`")]
    #[diagnostic(code(rs_shell::missing_named_argument_value))]
    MissingNamedArgumentValue {
        name: String,
        #[label("missing named argument value")]
        span: Span,
    },
}

impl ShellError {
    pub fn report_eprintln(self, line: &str) {
        let report_err = |err: ShellError| {
            let report =
                Report::new(err).with_source_code(NamedSource::new("line", line.to_string()));
            eprintln!("{report:?}");
        };

        if let ShellError::Multiple(errors) = self {
            for err in errors {
                report_err(err);
            }
        } else {
            report_err(self);
        }
    }

    pub fn type_mismatch(
        expected: impl Into<String>,
        actual: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            actual: actual.into(),
            span,
        }
    }
}
