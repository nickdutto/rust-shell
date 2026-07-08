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
use crate::parser::CommandNode;
use crate::parser::word::words_to_string;
use crate::shell::config::Config;
use crate::shell::shell_state::ShellState;
use reedline::ExternalPrinter;
use std::process::Child;
use std::sync::{Arc, RwLock};

pub const BUILTIN_COMMANDS: &[&str] = &[
    "cd", "complete", "echo", "declare", "exit", "history", "jobs", "pwd", "theme", "type",
];

pub fn run_command(
    command_node: CommandNode,
    config: Arc<Config>,
    shell_state: Arc<RwLock<ShellState>>,
    printer: &ExternalPrinter<String>,
    mut io_streams: IoStreams,
) -> Option<Child> {
    if command_node.redirection.mode != RedirectionMode::Nothing
        && !command_node.redirection.path.is_empty()
    {
        let file = initialise_writer_file(
            command_node.redirection.mode.clone(),
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

    match cmd.as_str() {
        "cd" => {
            handle_cd(args, shell_state, io_streams);
            None
        }
        "complete" => {
            handle_complete(args, shell_state, io_streams);
            None
        }
        "declare" => {
            handle_declare(args, shell_state, io_streams);
            None
        }
        "echo" => {
            handle_echo(args, io_streams);
            None
        }
        "exit" => {
            handle_exit(shell_state, io_streams);
            None
        }
        "history" => {
            handle_history(args, shell_state, io_streams);
            None
        }
        "jobs" => {
            handle_jobs(args, shell_state, io_streams);
            None
        }
        "pwd" => {
            handle_pwd(io_streams);
            None
        }
        "theme" => {
            handle_theme(args, config, io_streams);
            None
        }
        "type" => {
            handle_type(&cmd, io_streams);
            None
        }
        _cmd => handle_executable(&cmd, args, shell_state, io_streams, printer.clone()),
    }
}
