use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::network::http_client::HttpClient;
use crate::network::http_method::HttpMethod;
use crate::parser::span::{Span, Spanned};
use crate::shell::shell_state::ShellState;
use std::error::Error;
use std::io::Write;
use std::sync::{Arc, RwLock};
use url::Url;

pub struct Http;

impl Command for Http {
    fn name(&self) -> &'static str {
        "http"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn run(
        &self,
        cmd: Spanned<String>,
        args: Vec<Spanned<String>>,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        _shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut args_iter = args.into_iter();

        let http_method_res = Self::parse_arg(
            "HttpMethod",
            args_iter.next(),
            cmd.span,
            |s| HttpMethod::parse(&s),
            HttpMethod::available_methods,
            String::new,
        );

        let url_res = Self::parse_arg(
            "Url",
            args_iter.next(),
            cmd.span,
            |s| Url::parse(&s),
            String::new,
            String::new,
        );

        let (Ok(http_method), Ok(url)) = (&http_method_res, &url_res) else {
            let mut errors = vec![];
            errors.extend(http_method_res.err());
            errors.extend(url_res.err());
            return Err(ShellError::Multiple(errors));
        };

        match HttpClient::send_request(http_method, url) {
            Ok(res) => {
                writeln!(io_streams.output, "{res}")?;
            }
            Err(e) => {
                writeln!(io_streams.error, "{e}")?;
                return Ok(CommandData::ExitCode(ExitCode::FAILURE));
            }
        }

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}

impl Http {
    fn parse_arg<T, E, Parser, ParserHelp, ErrorHelp>(
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
}
