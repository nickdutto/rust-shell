use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;

pub struct Sys;

impl Command for Sys {
    fn name(&self) -> &'static str {
        "sys"
    }

    fn description(&self) -> &'static str {
        "System information"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name()).category(Category::System)
    }

    fn run(
        &self,
        _call: Call,
        _engine_state: &EngineState,
        _io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
