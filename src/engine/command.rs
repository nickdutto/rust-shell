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
use crate::engine::process::ProcessHandle;
use crate::io::redirection::{RedirectionMode, initialise_writer_file};
use crate::io::stream::{IoStreams, OutputStream};
use crate::parser::command_node::CommandNode;
use crate::parser::word::words_to_string;
use crate::shell::config::Config;
use crate::shell::shell_state::ShellState;
use std::sync::{Arc, RwLock};

pub const BUILTIN_COMMANDS: &[&str] = &[
    "cd", "complete", "echo", "declare", "exit", "history", "jobs", "pwd", "theme", "type",
];

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

    let needs_thread = matches!(io_streams.output, OutputStream::Pipe(_));

    // TODO: need to actually handle exit codes in builtins
    match cmd.as_str() {
        "cd" => {
            handle_cd(args, shell_state, io_streams);
            ProcessHandle::Immediate(0)
        }
        "complete" => {
            handle_complete(args, shell_state, io_streams);
            ProcessHandle::Immediate(0)
        }
        "declare" => {
            handle_declare(args, shell_state, io_streams);
            ProcessHandle::Immediate(0)
        }
        "exit" => {
            handle_exit(shell_state, io_streams);
            ProcessHandle::Immediate(0)
        }

        "echo" => ProcessHandle::run_producer(
            Box::new(move || {
                handle_echo(args, io_streams);
                0
            }),
            needs_thread,
        ),
        "history" => ProcessHandle::run_producer(
            Box::new(move || {
                handle_history(args, shell_state, io_streams);
                0
            }),
            needs_thread,
        ),
        "jobs" => ProcessHandle::run_producer(
            Box::new(move || {
                handle_jobs(args, shell_state, io_streams);
                0
            }),
            needs_thread,
        ),
        "pwd" => ProcessHandle::run_producer(
            Box::new(move || {
                handle_pwd(io_streams);
                0
            }),
            needs_thread,
        ),
        "theme" => ProcessHandle::run_producer(
            Box::new(move || {
                handle_theme(args, config, io_streams);
                0
            }),
            needs_thread,
        ),
        "type" => ProcessHandle::run_producer(
            Box::new(move || {
                handle_type(&cmd, io_streams);
                0
            }),
            needs_thread,
        ),
        _cmd => match handle_executable(&cmd, args, io_streams) {
            Ok(child) => {
                if let Some(job_id) = current_job_id
                    && let Some(job) = shell_state
                    .write()
                    .unwrap()
                    .background_jobs
                    .iter_mut()
                    .find(|job| job.id() == job_id)
                {
                    job.pids.push(child.id());
                }

                ProcessHandle::External(child)
            }
            Err(exit_code) => ProcessHandle::Immediate(exit_code),
        },
    }
}
