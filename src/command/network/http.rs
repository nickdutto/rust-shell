use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::network::http_client::HttpClient;
use crate::network::http_method::HttpMethod;
use crate::parser::argument::parse_arg;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
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

        let http_method_res = parse_arg(
            "HttpMethod",
            args_iter.next(),
            cmd.span,
            |s| HttpMethod::parse(&s),
            HttpMethod::available_methods,
            String::new,
        );

        let url_res = parse_arg(
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
