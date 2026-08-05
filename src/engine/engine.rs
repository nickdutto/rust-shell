use crate::config::Config;
use crate::engine::exit::ExitCode;
use crate::engine::expansion::expand_command_node_values;
use crate::engine::process::ProcessHandle;
use crate::engine::router::CommandRouter;
use crate::error::shell_error::ShellError;
use crate::io::stream::{
    InputStream, IoStreams, OutputStream, background_io_streams, pipe_io_streams,
};
use crate::parser::Parser;
use crate::parser::command_node::CommandNode;
use crate::parser::error::collect_parser_errors;
use crate::parser::lexer::lex;
use crate::parser::statement::Statement;
use crate::shell::background_jobs::BackgroundJob;
use crate::shell::shell_state::ShellState;
use miette::{NamedSource, Report};
use reedline::ExternalPrinter;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Engine {
    config: Arc<Config>,
    command_router: Arc<CommandRouter>,
    shell_state: Arc<RwLock<ShellState>>,
    printer: ExternalPrinter<String>,
}

impl Engine {
    pub fn new(
        config: Arc<Config>,
        command_router: Arc<CommandRouter>,
        shell_state: Arc<RwLock<ShellState>>,
        printer: ExternalPrinter<String>,
    ) -> Self {
        Self {
            config,
            command_router,
            shell_state,
            printer,
        }
    }

    pub fn run_line(&self, line: &str) {
        let tokens = lex(line);

        let errors = collect_parser_errors(&tokens);
        if !errors.errors.is_empty() {
            ShellError::ParserMulti(errors).report_eprintln(line);
            return;
        }

        let statements = Parser::new(tokens).parse_statements();
        for statement in statements {
            if let Err(err) = self.run_statement(statement, None, line, IoStreams::new()) {
                err.report_eprintln(line);
            }
        }
    }

    fn run_statement(
        &self,
        statement: Statement,
        current_job_id: Option<usize>,
        line: &str,
        io_streams: IoStreams,
    ) -> Result<ExitCode, ShellError> {
        match statement {
            Statement::Sequential { left, right } => {
                self.run_sequential(*left, *right, current_job_id, line, io_streams)
            }

            Statement::And { left, right } => {
                self.run_and(*left, *right, current_job_id, line, io_streams)
            }

            Statement::Pipeline(command_nodes) => {
                self.run_pipeline(command_nodes, current_job_id, line, &io_streams)
            }

            Statement::Background(inner_statement) => {
                Ok(self.run_background(*inner_statement, line.to_owned()))
            }

            Statement::Command(command_node) => self
                .run_command(command_node, current_job_id, io_streams)
                .wait(),
        }
    }

    fn run_sequential(
        &self,
        left: Statement,
        right: Statement,
        current_job_id: Option<usize>,
        line: &str,
        io_streams: IoStreams,
    ) -> Result<ExitCode, ShellError> {
        if let Err(err) = self.run_statement(
            left,
            current_job_id,
            line,
            io_streams.try_clone().unwrap_or_default(),
        ) {
            err.report_eprintln(line);
        }

        self.run_statement(right, current_job_id, line, io_streams)
    }

    fn run_and(
        &self,
        left: Statement,
        right: Statement,
        current_job_id: Option<usize>,
        line: &str,
        io_streams: IoStreams,
    ) -> Result<ExitCode, ShellError> {
        let left_exit_code = self.run_statement(
            left,
            current_job_id,
            line,
            io_streams.try_clone().unwrap_or_default(),
        )?;

        if left_exit_code == ExitCode::SUCCESS {
            self.run_statement(right, current_job_id, line, io_streams)
        } else {
            Ok(left_exit_code)
        }
    }

    fn run_pipeline(
        &self,
        command_nodes: Vec<CommandNode>,
        current_job_id: Option<usize>,
        line: &str,
        io_streams: &IoStreams,
    ) -> Result<ExitCode, ShellError> {
        let mut current_input = InputStream::Stdin;
        let mut handles: Vec<ProcessHandle> = vec![];
        let mut iter = command_nodes.into_iter().peekable();

        while let Some(command_node) = iter.next() {
            let io_streams_clone = io_streams.try_clone().unwrap_or_default();

            let (piped_io_streams, next_input) = match pipe_io_streams(
                current_input,
                io_streams_clone.output,
                io_streams_clone.error,
                &mut iter,
            ) {
                Ok(streams) => streams,
                Err(exit_code) => {
                    for handle in handles {
                        let _ = handle.wait();
                    }
                    return Ok(exit_code);
                }
            };

            let handle = self.run_command(command_node, current_job_id, piped_io_streams);

            if let ProcessHandle::Immediate(Err(err)) = handle {
                for h in handles {
                    let _ = h.wait();
                }
                return Err(err);
            }

            handles.push(handle);
            current_input = next_input;
        }

        let mut final_exit_code = ExitCode::SUCCESS;
        let mut errors = vec![];

        for handle in handles {
            match handle.wait() {
                Ok(exit_code) => final_exit_code = exit_code,
                Err(err) => errors.push(err),
            }
        }

        if !errors.is_empty() {
            let first_err = errors.remove(0);

            for err in errors {
                err.report_eprintln(line);
            }

            Err(first_err)
        } else {
            Ok(final_exit_code)
        }
    }

    fn run_background(&self, statement: Statement, line: String) -> ExitCode {
        let cmd_string = format!("{} &", statement.to_statement_string());

        let job_id = self
            .shell_state
            .write()
            .unwrap()
            .background_jobs
            .add_job(vec![], cmd_string.clone());

        let _ = self
            .printer
            .print(BackgroundJob::format_job_running(job_id));

        let runner_clone = self.clone();
        let shell_state_clone = Arc::clone(&self.shell_state);

        std::thread::spawn(move || {
            let Ok((bg_io_streams, out_reader_printer, err_reader_printer)) =
                background_io_streams(&runner_clone.printer)
            else {
                return;
            };

            if let Err(error) =
                runner_clone.run_statement(statement, Some(job_id), &line, bg_io_streams)
            {
                let report_err = |err: ShellError| {
                    let report = Report::new(err)
                        .with_source_code(NamedSource::new("line", cmd_string.clone()));
                    let formatted = format!("{report:?}");

                    for report_line in formatted.lines() {
                        let trimmed = report_line.trim_end();
                        if !trimmed.is_empty() {
                            let _ = runner_clone.printer.print(trimmed.to_string());
                        }
                    }
                };

                if let ShellError::Multiple(errors) = error {
                    for err in errors {
                        report_err(err);
                    }
                } else {
                    report_err(error);
                }
            }

            let _ = out_reader_printer.join();
            let _ = err_reader_printer.join();

            if let Some(output) = shell_state_clone
                .write()
                .unwrap()
                .background_jobs
                .complete_job(job_id)
            {
                let _ = runner_clone.printer.print(output);
            }
        });

        ExitCode::SUCCESS
    }

    fn run_command(
        &self,
        command_node: CommandNode,
        current_job_id: Option<usize>,
        mut io_streams: IoStreams,
    ) -> ProcessHandle {
        io_streams.apply_redirection(
            command_node.redirection,
            &self.shell_state.read().unwrap().variables,
        );

        let (cmd_name, args) =
            expand_command_node_values(command_node.cmd, command_node.args, &self.shell_state);

        let needs_thread = self.command_router.is_builtin(&cmd_name.item)
            && matches!(io_streams.output, OutputStream::Pipe(_));

        self.command_router.dispatch(
            cmd_name,
            args,
            needs_thread,
            current_job_id,
            Arc::clone(&self.config),
            Arc::clone(&self.shell_state),
            io_streams,
        )
    }
}
