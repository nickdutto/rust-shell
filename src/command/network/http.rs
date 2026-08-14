use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::network::http_client::HttpClient;
use crate::network::http_method::HttpMethod;
use crate::parser::argument::ParsedArguments;
use crate::parser::span::{Span, Spanned};
use crate::parser::syntax_shape::SyntaxShape;
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

    fn signature(&self) -> Signature {
        Signature::new("http")
            .required_positional("METHOD", SyntaxShape::String, "HTTP Method")
            .required_positional("URL", SyntaxShape::String, "URL to fetch from")
    }

    fn run(
        &self,
        cmd: Spanned<String>,
        args: ParsedArguments,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        _shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let http_method_res = HttpMethod::parse(&args.req::<String>(0)?)
            .map_err(|err| map_shell_error(err, cmd.span, String::new()));

        let url_res = Url::parse(&args.req::<String>(1)?)
            .map_err(|err| map_shell_error(err, cmd.span, String::new()));

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

fn map_shell_error<E: Error>(err: E, span: Span, help: String) -> ShellError {
    ShellError::CommandArgument {
        span,
        help,
        label: err.to_string(),
    }
}
