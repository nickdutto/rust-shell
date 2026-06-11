use crate::command::builtin::cd::handle_cd;
use crate::command::builtin::complete::handle_complete;
use crate::command::builtin::echo::handle_echo;
use crate::command::builtin::executable::handle_executable;
use crate::command::builtin::exit::handle_exit;
use crate::command::builtin::history::handle_history;
use crate::command::builtin::jobs::handle_jobs;
use crate::command::builtin::pwd::handle_pwd;
use crate::command::builtin::type_cmd::handle_type;
use crate::io::tokenize::{Tokens, tokenize_arguments};
use crate::io::writer::initialise_writer_file;
use crate::shell::shell_state::ShellState;
use std::sync::{Arc, RwLock};

pub const BUILTIN_COMMANDS: &[&str] = &[
    "cd", "complete", "echo", "exit", "history", "jobs", "pwd", "type",
];

pub enum Command {
    Cd(Tokens),
    Complete(Tokens),
    Echo(Tokens),
    Executable(Tokens),
    Exit,
    History(Tokens),
    Jobs(Tokens),
    Pwd(Tokens),
    Type(Tokens),
}

impl Command {
    pub fn parse_command(input: &str) -> Self {
        let tokens = tokenize_arguments(input.trim());

        match tokens.command.as_str() {
            "cd" => Command::Cd(tokens),
            "complete" => Command::Complete(tokens),
            "echo" => Command::Echo(tokens),
            "exit" => Command::Exit,
            "history" => Command::History(tokens),
            "jobs" => Command::Jobs(tokens),
            "pwd" => Command::Pwd(tokens),
            "type" => Command::Type(tokens),
            _ => Command::Executable(tokens),
        }
    }

    pub fn run_command(self, shell_state: Arc<RwLock<ShellState>>) {
        self.initialise_redirection_file();

        match self {
            Command::Cd(tokens) => handle_cd(tokens),
            Command::Complete(tokens) => handle_complete(tokens, &mut shell_state.write().unwrap()),
            Command::Echo(tokens) => handle_echo(tokens),
            Command::Executable(tokens) => handle_executable(tokens, shell_state),
            Command::Exit => handle_exit(shell_state),
            Command::History(tokens) => handle_history(tokens, shell_state),
            Command::Jobs(tokens) => handle_jobs(tokens, shell_state),
            Command::Pwd(tokens) => handle_pwd(tokens),
            Command::Type(tokens) => handle_type(tokens),
        }
    }

    fn initialise_redirection_file(&self) {
        let tokens_ref = match self {
            Command::Cd(tokens)
            | Command::Complete(tokens)
            | Command::Echo(tokens)
            | Command::Executable(tokens)
            | Command::History(tokens)
            | Command::Jobs(tokens)
            | Command::Pwd(tokens)
            | Command::Type(tokens) => Some(tokens),
            Command::Exit => None,
        };

        if let Some(tokens) = tokens_ref
            && let Some(redirection) = &tokens.redirection
        {
            initialise_writer_file(redirection);
        }
    }
}
