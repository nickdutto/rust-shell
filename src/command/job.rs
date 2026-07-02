use crate::command::Command;
use crate::io::stream::{InputStream, IoStreams, OutputStream};
use crate::parser::tokenize::tokenize_arguments;
use crate::shell::config::Config;
use crate::shell::shell_state::ShellState;
use crate::shell::variables::Variables;
use reedline::ExternalPrinter;
use std::sync::{Arc, RwLock};

pub enum Job {
    Single(Command),
    Pipeline(Vec<Command>),
}

impl Job {
    pub fn parse_line(input: &str, variables: &Variables) -> Option<Job> {
        let pipeline_tokens = tokenize_arguments(input.trim(), variables);

        if pipeline_tokens.is_empty() {
            return None;
        }

        let mut commands = vec![];
        for tokens in pipeline_tokens {
            let command = match tokens.command.as_str() {
                "cd" => Command::Cd(tokens),
                "complete" => Command::Complete(tokens),
                "declare" => Command::Declare(tokens),
                "echo" => Command::Echo(tokens),
                "exit" => Command::Exit,
                "history" => Command::History(tokens),
                "jobs" => Command::Jobs(tokens),
                "pwd" => Command::Pwd,
                "theme" => Command::Theme(tokens),
                "type" => Command::Type(tokens),
                _ => Command::Executable(tokens),
            };
            commands.push(command);
        }

        if commands.len() == 1 {
            Some(Job::Single(commands.pop().unwrap()))
        } else {
            Some(Job::Pipeline(commands))
        }
    }

    pub fn run(
        self,
        config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        printer: ExternalPrinter<String>,
    ) {
        match self {
            Job::Single(command) => {
                let streams = IoStreams {
                    input: InputStream::Stdin,
                    output: OutputStream::Stdout,
                    error: OutputStream::Stderr,
                };

                if let Some(mut child) = command.run_command(config, shell_state, printer, streams)
                {
                    let _ = child.wait();
                }
            }
            Job::Pipeline(commands) => {
                let mut current_input = InputStream::Stdin;
                let mut active_children = vec![];
                let mut iter = commands.into_iter().peekable();

                while let Some(command) = iter.next() {
                    let is_last = iter.peek().is_none();

                    let (next_input, current_output) = if !is_last {
                        let (read_end, write_end) = std::io::pipe().unwrap();
                        (InputStream::Pipe(read_end), OutputStream::Pipe(write_end))
                    } else {
                        (InputStream::Stdin, OutputStream::Stdout)
                    };

                    let streams = IoStreams {
                        input: current_input,
                        output: current_output,
                        error: OutputStream::Stderr,
                    };

                    if let Some(child) = command.run_command(
                        Arc::clone(&config),
                        Arc::clone(&shell_state),
                        printer.clone(),
                        streams,
                    ) {
                        active_children.push(child);
                    }

                    current_input = next_input;
                }

                for mut child in active_children {
                    let _ = child.wait();
                }
            }
        }
    }
}
