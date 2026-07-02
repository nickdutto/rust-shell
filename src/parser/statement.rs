use crate::command::Command;
use crate::io::stream::{InputStream, IoStreams, OutputStream};
use crate::parser::parser::{AstElement, parse_ast_elements};
use crate::shell::config::Config;
use crate::shell::shell_state::ShellState;
use crate::shell::variables::Variables;
use reedline::ExternalPrinter;
use std::sync::{Arc, RwLock};

pub enum Statement {
    Pipeline(Vec<Command>),
}

impl Statement {
    pub fn parse_line(line: &str, variables: &Variables) -> Option<Statement> {
        let elements = parse_ast_elements(line.trim(), variables);
        Self::parse_elements(elements)
    }

    fn parse_elements(elements: Vec<AstElement>) -> Option<Statement> {
        if elements.is_empty() {
            return None;
        }

        let mut commands = vec![];
        for element in elements {
            if let AstElement::Command(command_node) = element {
                let command = Command::from_command_node(command_node);
                commands.push(command);
            }
        }

        if commands.is_empty() {
            None
        } else {
            Some(Statement::Pipeline(commands))
        }
    }

    pub fn run(
        self,
        config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        printer: ExternalPrinter<String>,
    ) {
        match self {
            Statement::Pipeline(commands) => {
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
