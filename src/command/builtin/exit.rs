use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::io::{Write, stdout};
use std::process;
use std::sync::{Arc, RwLock};

pub struct Exit;

impl Command for Exit {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn run(
        &self,
        _cmd: Spanned<String>,
        args: Vec<Spanned<String>>,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        if let Err(e) = shell_state
            .write()
            .unwrap()
            .history
            .exit_save_history_file()
        {
            writeln!(io_streams.error, "{e}")?;
        }

        stdout().flush()?;

        let exit_code = args
            .first()
            .and_then(|s| s.item.parse::<i32>().ok())
            .unwrap_or(0);

        process::exit(exit_code);
    }
}
