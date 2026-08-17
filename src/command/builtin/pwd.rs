use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::argument::ParsedArguments;
use crate::parser::span::Spanned;
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
        Signature::new(self.name())
    }

    fn run(
        &self,
        _cmd: Spanned<String>,
        _args: ParsedArguments,
        _job_id: Option<usize>,
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
