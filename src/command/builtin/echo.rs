use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandError, CommandType};
use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::io::Write as IoWrite;
use std::sync::{Arc, RwLock};

pub struct Echo;

impl Command for Echo {
    fn name(&self) -> &'static str {
        "echo"
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
        _shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, CommandError> {
        writeln!(
            io_streams.output,
            "{}",
            args.iter()
                .map(|s| s.item.as_str())
                .collect::<Vec<&str>>()
                .join(" ")
                .trim()
        )?;
        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
