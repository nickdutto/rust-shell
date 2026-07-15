use crate::engine::exit::ExitCode;
use crate::engine::process::ProcessHandle;
use crate::engine::router::CommandRouter;
use crate::io::redirection::{RedirectionMode, initialise_writer_file};
use crate::io::stream::{IoStreams, OutputStream};
use crate::parser::command_node::{CommandNode, Redirection};
use crate::parser::word::words_to_string;
use crate::shell::config::Config;
use crate::shell::shell_state::ShellState;
use std::io;
use std::process::Child;
use std::sync::{Arc, RwLock};
use thiserror::Error;

pub const BUILTIN_COMMANDS: &[&str] = &[
    "cd", "complete", "echo", "declare", "exit", "history", "jobs", "pwd", "theme", "type",
];

#[derive(Error, Debug)]
pub enum CommandError {
    #[error(transparent)]
    Io(#[from] io::Error),
}

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
        cmd: &str,
        args: Vec<String>,
        job_id: Option<usize>,
        config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        io_streams: IoStreams,
    ) -> Result<CommandData, CommandError>;
}

pub fn run_command(
    command_node: CommandNode,
    current_job_id: Option<usize>,
    config: Arc<Config>,
    command_router: &Arc<CommandRouter>,
    shell_state: Arc<RwLock<ShellState>>,
    mut io_streams: IoStreams,
) -> ProcessHandle {
    init_redirection_io_streams(command_node.redirection, &shell_state, &mut io_streams);

    let command_name = words_to_string(command_node.cmd, &shell_state.read().unwrap().variables);
    let needs_thread = matches!(io_streams.output, OutputStream::Pipe(_));

    let mut args = vec![];
    for arg in command_node.args {
        args.push(words_to_string(arg, &shell_state.read().unwrap().variables));
    }

    command_router.dispatch(
        command_name,
        args,
        needs_thread,
        current_job_id,
        config,
        shell_state,
        io_streams,
    )
}

pub fn init_redirection_io_streams(
    redirection: Redirection,
    shell_state: &Arc<RwLock<ShellState>>,
    io_streams: &mut IoStreams,
) {
    if redirection.mode != RedirectionMode::Nothing && !redirection.path.is_empty() {
        let file = initialise_writer_file(
            &redirection.mode,
            &words_to_string(redirection.path, &shell_state.read().unwrap().variables),
        );

        match redirection.mode {
            RedirectionMode::Out | RedirectionMode::OutAppend => {
                io_streams.output = OutputStream::File(file);
            }
            RedirectionMode::Error | RedirectionMode::ErrorAppend => {
                io_streams.error = OutputStream::File(file);
            }
            RedirectionMode::Nothing => {
                io_streams.output = OutputStream::Stdout;
                io_streams.error = OutputStream::Stderr;
            }
        }
    }
}
