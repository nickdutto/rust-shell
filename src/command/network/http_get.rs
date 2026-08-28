use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::network::http_client::HttpClient;
use crate::network::http_method::HttpMethod;
use crate::value::span::Spanned;
use crate::value::syntax_shape::SyntaxShape;
use std::io::Write;
use url::Url;

pub struct HttpGet;

impl Command for HttpGet {
    fn name(&self) -> &'static str {
        "http get"
    }

    fn description(&self) -> &'static str {
        "Run http get request for a URL"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::Network)
            .required_positional("url", SyntaxShape::String, "url to fetch content from")
    }

    fn run(
        &self,
        call: Call,
        engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let call_span = call.call_span();

        let url_arg = call.req::<Spanned<String>>(0)?;
        let url = Url::parse(&url_arg.item).map_err(|err| ShellError::CommandArgument {
            span: url_arg.span,
            help: String::new(),
            label: err.to_string(),
        })?;

        let response = HttpClient::send_request(&HttpMethod::Get, &url, engine_state, call_span)?
            .body_mut()
            .read_to_string()?;

        writeln!(io_streams.output, "{response}")?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
