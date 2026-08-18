use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::syntax_shape::SyntaxShape;
use std::io::Write as IoWrite;

pub struct Echo;

impl Command for Echo {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name()).category(Category::Core).rest(
            "message",
            SyntaxShape::String,
            "text to write",
        )
    }

    fn run(
        &self,
        call: Call,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let message = call.rest_strings().collect::<Vec<_>>().join(" ");

        writeln!(io_streams.output, "{message}")?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
