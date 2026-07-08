use crate::engine::command::run_command;
use crate::io::stream::IoStreams;
use crate::parser::lexer::lex;
use crate::parser::{Parser, Statement};
use crate::shell::config::Config;
use crate::shell::shell_state::ShellState;
use reedline::ExternalPrinter;
use std::sync::{Arc, RwLock};

pub fn run_line(
    line: &str,
    config: Arc<Config>,
    shell_state: Arc<RwLock<ShellState>>,
    printer: ExternalPrinter<String>,
) {
    run_statements(
        Parser::new(lex(line)).parse_statements(),
        config,
        shell_state,
        printer,
    );
}

pub fn run_statements(
    statements: Vec<Statement>,
    config: Arc<Config>,
    shell_state: Arc<RwLock<ShellState>>,
    printer: ExternalPrinter<String>,
) {
    for statement in statements {
        run_statement(
            statement,
            Arc::clone(&config),
            Arc::clone(&shell_state),
            &printer,
        );
    }
}

pub fn run_statement(
    statement: Statement,
    config: Arc<Config>,
    shell_state: Arc<RwLock<ShellState>>,
    printer: &ExternalPrinter<String>,
) {
    match statement {
        Statement::Command(command_node) => {
            let io_streams = IoStreams::new();
            run_command(command_node, config, shell_state, printer, io_streams);
        }
        Statement::Background(statement) => {
            todo!()
        }
        Statement::And { left, right } => {
            todo!()
        }
        Statement::Pipeline(command_nodes) => {
            todo!()
        }
        Statement::Sequence { left, right } => {
            todo!()
        }
    };
}
