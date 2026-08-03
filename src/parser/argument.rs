use crate::error::shell_error::ShellError;
use crate::parser::span::{Span, Spanned};
use std::error::Error;

pub fn parse_arg<T, E, Parser, ParserHelp, ErrorHelp>(
    arg_name: &str,
    raw_arg: Option<Spanned<String>>,
    cmd_span: Span,
    parser: Parser,
    parse_help: ParserHelp,
    error_help: ErrorHelp,
) -> Result<T, ShellError>
where
    E: Error,
    Parser: FnOnce(String) -> Result<T, E>,
    ParserHelp: FnOnce() -> String,
    ErrorHelp: FnOnce() -> String,
{
    if let Some(arg) = raw_arg {
        parser(arg.item).map_err(|e| ShellError::CommandArgument {
            span: arg.span,
            help: parse_help(),
            label: e.to_string(),
        })
    } else {
        Err(ShellError::CommandArgument {
            span: cmd_span,
            help: error_help(),
            label: format!("Empty `{arg_name}` argument"),
        })
    }
}
