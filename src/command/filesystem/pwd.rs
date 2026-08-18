use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use std::env;
use std::io::Write;

pub struct Pwd;

impl Command for Pwd {
    fn name(&self) -> &'static str {
        "pwd"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name()).category(Category::FileSystem)
    }

    fn run(
        &self,
        _call: Call,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut final_exit_code = ExitCode::SUCCESS;

        match env::current_dir() {
            Ok(path) => {
                writeln!(io_streams.output, "{}", path.display().to_string().trim())?;
            }
            Err(e) => {
                writeln!(io_streams.error, "{e}")?;
                final_exit_code = ExitCode::FAILURE;
            }
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}
