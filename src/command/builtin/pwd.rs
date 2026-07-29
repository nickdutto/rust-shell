use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandError, CommandType};
use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::env;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Pwd;

impl Command for Pwd {
    fn name(&self) -> &'static str {
        "pwd"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn run(
        &self,
        _cmd: Spanned<String>,
        _args: Vec<Spanned<String>>,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        _shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, CommandError> {
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
