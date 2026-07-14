use crate::command::builtin::cd::Cd;
use crate::command::builtin::complete::Complete;
use crate::command::builtin::declare::Declare;
use crate::command::builtin::echo::Echo;
use crate::command::builtin::executable::Executable;
use crate::command::builtin::exit::Exit;
use crate::command::builtin::history::History;
use crate::command::builtin::jobs::Jobs;
use crate::command::builtin::pwd::Pwd;
use crate::command::builtin::theme::Theme;
use crate::command::builtin::type_cmd::TypeCmd;
use crate::engine::exit::ExitCode;
use crate::engine::process::ProcessHandle;
use crate::io::redirection::{RedirectionMode, initialise_writer_file};
use crate::io::stream::{IoStreams, OutputStream};
use crate::parser::command_node::CommandNode;
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
    shell_state: Arc<RwLock<ShellState>>,
    mut io_streams: IoStreams,
) -> ProcessHandle {
    if command_node.redirection.mode != RedirectionMode::Nothing
        && !command_node.redirection.path.is_empty()
    {
        let file = initialise_writer_file(
            &command_node.redirection.mode,
            &words_to_string(
                command_node.redirection.path,
                &shell_state.read().unwrap().variables,
            ),
        );

        match command_node.redirection.mode {
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

    let cmd = words_to_string(command_node.cmd, &shell_state.read().unwrap().variables);

    let mut args = vec![];
    for arg in command_node.args {
        args.push(words_to_string(arg, &shell_state.read().unwrap().variables));
    }

    let needs_thread = matches!(io_streams.output, OutputStream::Pipe(_));

    match cmd.as_str() {
        "cd" => ProcessHandle::run_producer(
            Box::new(move || Cd.run(&cmd, args, None, config, shell_state, io_streams)),
            needs_thread,
        ),
        "complete" => ProcessHandle::run_producer(
            Box::new(move || Complete.run(&cmd, args, None, config, shell_state, io_streams)),
            needs_thread,
        ),
        "declare" => ProcessHandle::run_producer(
            Box::new(move || Declare.run(&cmd, args, None, config, shell_state, io_streams)),
            needs_thread,
        ),
        "exit" => ProcessHandle::run_producer(
            Box::new(move || Exit.run(&cmd, args, None, config, shell_state, io_streams)),
            needs_thread,
        ),
        "echo" => ProcessHandle::run_producer(
            Box::new(move || Echo.run(&cmd, args, None, config, shell_state, io_streams)),
            needs_thread,
        ),
        "history" => ProcessHandle::run_producer(
            Box::new(move || History.run(&cmd, args, None, config, shell_state, io_streams)),
            needs_thread,
        ),
        "jobs" => ProcessHandle::run_producer(
            Box::new(move || Jobs.run(&cmd, args, None, config, shell_state, io_streams)),
            needs_thread,
        ),
        "pwd" => ProcessHandle::run_producer(
            Box::new(move || Pwd.run(&cmd, args, None, config, shell_state, io_streams)),
            needs_thread,
        ),
        "theme" => ProcessHandle::run_producer(
            Box::new(move || Theme.run(&cmd, args, None, config, shell_state, io_streams)),
            needs_thread,
        ),
        "type" => ProcessHandle::run_producer(
            Box::new(move || TypeCmd.run(&cmd, args, None, config, shell_state, io_streams)),
            needs_thread,
        ),
        _cmd => ProcessHandle::run_producer(
            Box::new(move || {
                Executable.run(
                    &cmd,
                    args,
                    current_job_id,
                    config,
                    Arc::clone(&shell_state),
                    io_streams,
                )
            }),
            needs_thread,
        ),
    }
}
