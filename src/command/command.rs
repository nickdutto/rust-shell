use crate::command::builtin::cd::handle_cd;
use crate::command::builtin::complete::handle_complete;
use crate::command::builtin::declare::handle_declare;
use crate::command::builtin::echo::handle_echo;
use crate::command::builtin::executable::handle_executable;
use crate::command::builtin::exit::handle_exit;
use crate::command::builtin::history::handle_history;
use crate::command::builtin::jobs::handle_jobs;
use crate::command::builtin::pwd::handle_pwd;
use crate::command::builtin::theme::handle_theme;
use crate::command::builtin::type_cmd::handle_type;
use crate::io::redirection::{RedirectionMode, initialise_writer_file};
use crate::io::stream::{IoStreams, OutputStream};
use crate::io::tokenize::Tokens;
use crate::shell::shell_state::ShellState;
use reedline::ExternalPrinter;
use std::process::Child;
use std::sync::{Arc, RwLock};

pub const BUILTIN_COMMANDS: &[&str] = &[
    "cd", "complete", "echo", "declare", "exit", "history", "jobs", "pwd", "theme", "type",
];

pub enum Command {
    Cd(Tokens),
    Complete(Tokens),
    Declare(Tokens),
    Echo(Tokens),
    Executable(Tokens),
    Exit,
    History(Tokens),
    Jobs(Tokens),
    Pwd,
    Theme(Tokens),
    Type(Tokens),
}

impl Command {
    pub fn run_command(
        self,
        shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
        printer: ExternalPrinter<String>,
    ) -> Option<Child> {
        if let Some(tokens) = self.get_tokens()
            && let Some(redirection) = &tokens.redirection
        {
            let file = initialise_writer_file(redirection);
            match redirection.mode {
                RedirectionMode::Output | RedirectionMode::OutputAppend => {
                    io_streams.output = OutputStream::File(file);
                }
                RedirectionMode::Error | RedirectionMode::ErrorAppend => {
                    io_streams.error = OutputStream::File(file);
                }
            }
        }

        match self {
            Command::Cd(tokens) => {
                handle_cd(tokens, shell_state, io_streams);
                None
            }
            Command::Complete(tokens) => {
                handle_complete(tokens, shell_state, io_streams);
                None
            }
            Command::Declare(tokens) => {
                handle_declare(tokens, shell_state, io_streams);
                None
            }
            Command::Echo(tokens) => {
                handle_echo(tokens, io_streams);
                None
            }
            Command::Executable(tokens) => {
                handle_executable(tokens, shell_state, io_streams, printer)
            }
            Command::Exit => {
                handle_exit(shell_state, io_streams);
                None
            }
            Command::History(tokens) => {
                handle_history(tokens, shell_state, io_streams);
                None
            }
            Command::Jobs(tokens) => {
                handle_jobs(tokens, shell_state, io_streams);
                None
            }
            Command::Pwd => {
                handle_pwd(io_streams);
                None
            }
            Command::Theme(tokens) => {
                handle_theme(tokens, io_streams);
                None
            }
            Command::Type(tokens) => {
                handle_type(tokens, io_streams);
                None
            }
        }
    }

    fn get_tokens(&self) -> Option<&Tokens> {
        match self {
            Command::Cd(t)
            | Command::Complete(t)
            | Command::Declare(t)
            | Command::Echo(t)
            | Command::Executable(t)
            | Command::History(t)
            | Command::Jobs(t)
            | Command::Theme(t)
            | Command::Type(t) => Some(t),
            Command::Pwd | Command::Exit => None,
        }
    }
}
