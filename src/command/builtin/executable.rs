use crate::engine::command::{Command, CommandData, CommandError, CommandType};
use crate::engine::exit::ExitCode;
use crate::io::stream::{IoStreams, OutputStream};
use crate::shell::config::Config;
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
        cmd: &str,
        args: Vec<String>,
        job_id: Option<usize>,
        _config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        io_streams: IoStreams,
    ) -> Result<CommandData, CommandError> {
        if cmd.is_empty() {
            return Ok(CommandData::ExitCode(ExitCode::FAILURE));
        }

        let mut fallback_error = OutputStream::fallback_output_stream(&io_streams.error);

        let mut command_binding = std::process::Command::new(cmd);
        let command = command_binding
            .stdin(io_streams.input.into_stdio())
            .stdout(io_streams.output.into_stdio())
            .stderr(io_streams.error.into_stdio());

        match command.args(&args).spawn() {
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
                writeln!(fallback_error, "{cmd}: command not found")?;
                Ok(CommandData::ExitCode(ExitCode::NOT_FOUND))
            }
            Err(e) => {
                writeln!(fallback_error, "{cmd}: error executing command: {e}")?;
                Ok(CommandData::ExitCode(ExitCode::FAILURE))
            }
        }
    }
}
