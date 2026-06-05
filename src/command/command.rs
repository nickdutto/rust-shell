use crate::command::builtin::cd::handle_cd;
use crate::command::builtin::complete::handle_complete;
use crate::command::builtin::echo::handle_echo;
use crate::command::builtin::executable::handle_executable;
use crate::command::builtin::exit::handle_exit;
use crate::command::builtin::pwd::handle_pwd;
use crate::command::builtin::type_cmd::handle_type;
use crate::io::tokenize::{Tokens, tokenize_arguments};
use crate::io::writer::initialise_writer_file;
use crate::shell::ShellState;
use std::io::Write;

pub const BUILTIN_COMMANDS: &[&str] = &["cd", "complete", "echo", "exit", "pwd", "type"];

pub enum Command {
    Cd(Tokens),
    Complete(Tokens),
    Echo(Tokens),
    Executable(Tokens),
    Exit,
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
            "pwd" => Command::Pwd(tokens),
            "type" => Command::Type(tokens),
            _ => Command::Executable(tokens),
        }
    }

    pub fn run_command(
        self,
        shell_state: &mut ShellState,
        out_writer: &mut impl Write,
        err_writer: &mut impl Write,
    ) {
        self.initialise_redirection_file();

        match self {
            Command::Cd(tokens) => handle_cd(tokens, err_writer),
            Command::Complete(tokens) => {
                handle_complete(tokens, shell_state, out_writer, err_writer)
            }
            Command::Echo(tokens) => handle_echo(tokens, out_writer),
            Command::Executable(tokens) => handle_executable(tokens, out_writer, err_writer),
            Command::Exit => handle_exit(),
            Command::Pwd(tokens) => handle_pwd(tokens, out_writer),
            Command::Type(tokens) => handle_type(tokens, out_writer, err_writer),
        }
    }

    fn initialise_redirection_file(&self) {
        let tokens_ref = match self {
            Command::Cd(tokens)
            | Command::Complete(tokens)
            | Command::Echo(tokens)
            | Command::Executable(tokens)
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
