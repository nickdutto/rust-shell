use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::value::syntax_shape::SyntaxShape;

pub struct Http;

impl Command for Http {
    fn name(&self) -> &'static str {
        "http"
    }

    fn description(&self) -> &'static str {
        "Run http get request for a URL. Equivalent to `http get`"
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
        io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        if let Some(command) = engine_state.command_registry.get("http get") {
            command.run(call, engine_state, io_streams)?;
        }

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
