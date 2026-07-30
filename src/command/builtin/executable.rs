use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::{IoStreams, OutputStream};
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::io::ErrorKind;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Executable;

impl Command for Executable {
    fn name(&self) -> &'static str {
        "executable"
    }

    fn command_type(&self) -> CommandType {
        CommandType::External
    }

    fn run(
        &self,
        cmd: Spanned<String>,
        args: Vec<Spanned<String>>,
        job_id: Option<usize>,
        _config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        if cmd.item.is_empty() {
            return Ok(CommandData::ExitCode(ExitCode::FAILURE));
        }

        let mut fallback_error = OutputStream::fallback_output_stream(&io_streams.error);

        let mut command_binding = std::process::Command::new(&cmd.item);
        let command = command_binding
            .stdin(io_streams.input.into_stdio())
            .stdout(io_streams.output.into_stdio())
            .stderr(io_streams.error.into_stdio());

        match command.args(args.iter().map(|s| s.item.as_str())).spawn() {
            Ok(child) => {
                if let Some(jb_id) = job_id
                    && let Some(job) = shell_state
                    .write()
                    .unwrap()
                    .background_jobs
                    .iter_mut()
                    .find(|job| job.id() == jb_id)
                {
                    job.pids.push(child.id());
                }
                Ok(CommandData::Child(child))
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                writeln!(fallback_error, "{}: command not found", cmd.item)?;
                Ok(CommandData::ExitCode(ExitCode::NOT_FOUND))
            }
            Err(e) => {
                writeln!(fallback_error, "{}: error executing command: {e}", cmd.item)?;
                Ok(CommandData::ExitCode(ExitCode::FAILURE))
            }
        }
    }
}
