use crate::config::Config;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::process::Child;
use std::sync::{Arc, RwLock};

pub const BUILTIN_COMMANDS: &[&str] = &[
    "cd", "complete", "echo", "declare", "exit", "history", "jobs", "pwd", "theme", "type",
    "timezone", "lex", "explain", "http", "http get",
];

pub enum CommandData {
    Child(Child),
    ExitCode(ExitCode),
}

pub enum CommandType {
    Builtin,
    External,
}

impl CommandData {
    pub fn into_exit_code(self) -> ExitCode {
        match self {
            CommandData::Child(mut child) => child.wait().map_or(ExitCode::FAILURE, ExitCode::from),
            CommandData::ExitCode(code) => code,
        }
    }
}

pub trait Command {
    fn name(&self) -> &'static str;

    fn command_type(&self) -> CommandType;

    fn run(
        &self,
        cmd: Spanned<String>,
        args: Vec<Spanned<String>>,
        job_id: Option<usize>,
        config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        io_streams: IoStreams,
    ) -> Result<CommandData, ShellError>;
}
