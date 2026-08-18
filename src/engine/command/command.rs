use crate::engine::call::Call;
use crate::engine::command::signature::Signature;
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use std::fmt::{Display, Formatter};
use std::process::Child;

pub const BUILTIN_COMMANDS: &[&str] = &[
    "cd",
    "complete",
    "echo",
    "declare",
    "exit",
    "history",
    "jobs",
    "pwd",
    "theme",
    "type",
    "timezone",
    "ast",
    "lex",
    "explain",
    "help commands",
    "http",
    "http get",
];

pub enum CommandData {
    Child(Child),
    ExitCode(ExitCode),
}

#[derive(PartialEq)]
pub enum CommandType {
    Builtin,
    External,
}

impl Display for CommandType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandType::Builtin => write!(f, "builtin"),
            CommandType::External => write!(f, "external"),
        }
    }
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

    fn description(&self) -> &'static str {
        ""
    }

    fn command_type(&self) -> CommandType;

    fn signature(&self) -> Signature;

    fn run(
        &self,
        call: Call,
        engine_state: &EngineState,
        io_streams: IoStreams,
    ) -> Result<CommandData, ShellError>;
}
