use crate::engine::command::run_command;
use crate::io::stream::{InputStream, IoStreams, OutputStream};
use crate::parser::Parser;
use crate::parser::lexer::lex;
use crate::parser::statement::Statement;
use crate::shell::background_jobs::BackgroundJob;
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
            None,
            Arc::clone(&config),
            Arc::clone(&shell_state),
            &printer,
        );
    }
}

pub fn run_statement(
    statement: Statement,
    current_job_id: Option<usize>,
    config: Arc<Config>,
    shell_state: Arc<RwLock<ShellState>>,
    printer: &ExternalPrinter<String>,
) -> i32 {
    match statement {
        Statement::Command(command_node) => {
            let handle = run_command(
                command_node,
                current_job_id,
                config,
                shell_state,
                IoStreams::new(),
            );

            handle.wait()
        }

        Statement::Background(inner_statement) => {
            let shell_state_clone = Arc::clone(&shell_state);
            let printer_clone = printer.clone();

            let cmd_string = format!("{} &", inner_statement.to_statement_string());

            let job_id = shell_state
                .write()
                .unwrap()
                .background_jobs
                .add_job(vec![], cmd_string);
            let _ = printer.print(BackgroundJob::format_job_started(job_id));

            std::thread::spawn(move || {
                run_statement(
                    *inner_statement,
                    Some(job_id),
                    config,
                    shell_state,
                    &printer_clone,
                );

                if let Some(output) = shell_state_clone
                    .write()
                    .unwrap()
                    .background_jobs
                    .complete_job(job_id)
                {
                    let _ = printer_clone.print(output);
                }
            });

            0
        }

        Statement::And { left, right } => {
            let left_code = run_statement(
                *left,
                current_job_id,
                Arc::clone(&config),
                Arc::clone(&shell_state),
                printer,
            );

            if left_code == 0 {
                run_statement(*right, current_job_id, config, shell_state, printer)
            } else {
                left_code
            }
        }

        Statement::Pipeline(command_nodes) => {
            let mut current_input = InputStream::Stdin;
            let mut handles = vec![];
            let mut iter = command_nodes.into_iter().peekable();

            while let Some(command_node) = iter.next() {
                let (next_input, current_output) = if iter.peek().is_some() {
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

                let current_handle = run_command(
                    command_node,
                    current_job_id,
                    Arc::clone(&config),
                    Arc::clone(&shell_state),
                    streams,
                );
                handles.push(current_handle);

                current_input = next_input;
            }

            let mut final_code = 0;
            for handle in handles {
                final_code = handle.wait()
            }

            final_code
        }

        Statement::Sequential { left, right } => {
            run_statement(
                *left,
                current_job_id,
                Arc::clone(&config),
                Arc::clone(&shell_state),
                printer,
            );

            run_statement(*right, current_job_id, config, shell_state, printer)
        }
    }
}
