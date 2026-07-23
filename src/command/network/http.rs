use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandError, CommandType};
use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::network::http_client::HttpClient;
use crate::network::http_method::HttpMethod;
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
        _cmd: &str,
        args: Vec<String>,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        _shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, CommandError> {
        let mut args_iter = args.into_iter();
        let mut errors = vec![];

        let Some(method_arg) = args_iter.next() else {
            writeln!(
                io_streams.error,
                "http get: missing http method arg. Available methods: {}",
                HttpMethod::available_methods()
            )?;
            return Ok(CommandData::ExitCode(ExitCode::SYNTAX_ERROR));
        };

        let Some(url_arg) = args_iter.next() else {
            writeln!(io_streams.error, "http get: missing url arg")?;
            return Ok(CommandData::ExitCode(ExitCode::SYNTAX_ERROR));
        };

        let http_method = HttpMethod::parse(&method_arg)
            .map_err(|e| errors.push(e.to_string()))
            .ok();
        let url = Url::parse(&url_arg)
            .map_err(|e| errors.push(e.to_string()))
            .ok();

        let (Some(http_method), Some(url)) = (http_method, url) else {
            for e in errors {
                writeln!(io_streams.error, "{e}")?;
            }
            return Ok(CommandData::ExitCode(ExitCode::FAILURE));
        };

        match HttpClient::send_request(&http_method, &url) {
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
