use crate::config::Config;
use crate::engine::exit::ExitCode;
use crate::engine::process::ProcessHandle;
use crate::engine::router::CommandRouter;
use crate::error::shell_error::ShellError;
use crate::io::redirection::{RedirectionMode, initialise_writer_file};
use crate::io::stream::{InputStream, IoStreams, OutputStream};
use crate::parser::Parser;
use crate::parser::command_node::{CommandNode, Redirection};
use crate::parser::error::collect_parser_errors;
use crate::parser::lexer::lex;
use crate::parser::span::Spanned;
use crate::parser::statement::Statement;
use crate::parser::word::{Word, total_word_span, words_to_string};
use crate::shell::background_jobs::BackgroundJob;
use crate::shell::shell_state::ShellState;
use miette::{NamedSource, Report};
use reedline::ExternalPrinter;
use std::iter::Peekable;
use std::sync::{Arc, RwLock};
use std::vec::IntoIter;

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
            Self::report_error(ShellError::ParserMulti(errors), line);
            return;
        }

        let statements = Parser::new(tokens).parse_statements();
        for statement in statements {
            if let Err(err) = self.run_statement(statement, None, line) {
                Self::report_error(err, line);
            }
        }
    }

    pub fn run_statement(
        &self,
        statement: Statement,
        current_job_id: Option<usize>,
        line: &str,
    ) -> Result<ExitCode, ShellError> {
        match statement {
            Statement::Sequential { left, right } => {
                self.execute_sequential(*left, *right, current_job_id, line)
            }

            Statement::And { left, right } => self.execute_and(*left, *right, current_job_id, line),

            Statement::Pipeline(command_nodes) => {
                self.execute_pipeline(command_nodes, current_job_id, line)
            }

            Statement::Background(inner_statement) => {
                Ok(self.execute_background(*inner_statement, line.to_owned()))
            }

            Statement::Command(command_node) => self
                .run_command(command_node, current_job_id, IoStreams::new())
                .wait(),
        }
    }

    fn execute_sequential(
        &self,
        left: Statement,
        right: Statement,
        current_job_id: Option<usize>,
        line: &str,
    ) -> Result<ExitCode, ShellError> {
        if let Err(err) = self.run_statement(left, current_job_id, line) {
            Self::report_error(err, line);
        }
        self.run_statement(right, current_job_id, line)
    }

    fn execute_and(
        &self,
        left: Statement,
        right: Statement,
        current_job_id: Option<usize>,
        line: &str,
    ) -> Result<ExitCode, ShellError> {
        let left_exit_code = self.run_statement(left, current_job_id, line)?;

        if left_exit_code == ExitCode::SUCCESS {
            self.run_statement(right, current_job_id, line)
        } else {
            Ok(left_exit_code)
        }
    }

    fn execute_pipeline(
        &self,
        command_nodes: Vec<CommandNode>,
        current_job_id: Option<usize>,
        line: &str,
    ) -> Result<ExitCode, ShellError> {
        let mut current_input = InputStream::Stdin;
        let mut handles: Vec<ProcessHandle> = vec![];
        let mut iter = command_nodes.into_iter().peekable();

        while let Some(command_node) = iter.next() {
            let (io_streams, next_input) = match self.pipe_io_streams(current_input, &mut iter) {
                Ok(streams) => streams,
                Err(exit_code) => {
                    for handle in handles {
                        let _ = handle.wait();
                    }
                    return Ok(exit_code);
                }
            };

            let handle = self.run_command(command_node, current_job_id, io_streams);

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
                Self::report_error(err, line);
            }

            Err(first_err)
        } else {
            Ok(final_exit_code)
        }
    }

    fn execute_background(&self, statement: Statement, line: String) -> ExitCode {
        let runner_clone = self.clone();
        let shell_state_clone = Arc::clone(&self.shell_state);

        let cmd_string = format!("{} &", statement.to_statement_string());

        let job_id = self
            .shell_state
            .write()
            .unwrap()
            .background_jobs
            .add_job(vec![], cmd_string.clone());

        let _ = self
            .printer
            .print(BackgroundJob::format_job_started(job_id));

        std::thread::spawn(move || {
            if let Err(err) = runner_clone.run_statement(statement, Some(job_id), &line) {
                let report =
                    Report::new(err).with_source_code(NamedSource::new("line", cmd_string));

                let _ = runner_clone.printer.print(format!("{report:?}"));
            }

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

    fn pipe_io_streams(
        &self,
        current_input: InputStream,
        iter: &mut Peekable<IntoIter<CommandNode>>,
    ) -> Result<(IoStreams, InputStream), ExitCode> {
        let (next_input, current_output) = if iter.peek().is_some() {
            match std::io::pipe() {
                Ok((read_end, write_end)) => {
                    (InputStream::Pipe(read_end), OutputStream::Pipe(write_end))
                }
                Err(err) => {
                    let _ = self
                        .printer
                        .print(format!("shell: pipe creation failed: {err}"));

                    return Err(ExitCode::FAILURE);
                }
            }
        } else {
            (InputStream::Stdin, OutputStream::Stdout)
        };

        Ok((
            IoStreams {
                input: current_input,
                output: current_output,
                error: OutputStream::Stderr,
            },
            next_input,
        ))
    }

    pub fn run_command(
        &self,
        command_node: CommandNode,
        current_job_id: Option<usize>,
        mut io_streams: IoStreams,
    ) -> ProcessHandle {
        self.init_redirection_io_streams(command_node.redirection, &mut io_streams);

        let (cmd_name, args) = self.expand_command_node_values(command_node.cmd, command_node.args);
        let is_builtin = self.command_router.is_builtin(&cmd_name.item);
        let needs_thread = is_builtin && matches!(io_streams.output, OutputStream::Pipe(_));

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

    fn init_redirection_io_streams(&self, redirection: Redirection, io_streams: &mut IoStreams) {
        if redirection.mode != RedirectionMode::Nothing && !redirection.path.is_empty() {
            let file = initialise_writer_file(
                &redirection.mode,
                &words_to_string(
                    redirection.path,
                    &self.shell_state.read().unwrap().variables,
                ),
            );

            match redirection.mode {
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
    }

    fn expand_command_node_values(
        &self,
        node_cmd: Vec<Spanned<Word>>,
        node_args: Vec<Vec<Spanned<Word>>>,
    ) -> (Spanned<String>, Vec<Spanned<String>>) {
        let cmd_span = total_word_span(&node_cmd);
        let mut cmd_name = words_to_string(node_cmd, &self.shell_state.read().unwrap().variables);
        let mut args = vec![];

        if let Some(aliased_value) = self.shell_state.read().unwrap().aliases.get(&cmd_name) {
            let mut aliased_args = aliased_value
                .split_whitespace()
                .map(String::from)
                .collect::<Vec<String>>();

            if !aliased_args.is_empty() {
                cmd_name = aliased_args.remove(0);
                for aliased_arg in aliased_args {
                    args.push(Spanned::new(aliased_arg, cmd_span));
                }
            }
        }

        for arg in node_args {
            let arg_span = total_word_span(&arg);
            let arg_string = words_to_string(arg, &self.shell_state.read().unwrap().variables);

            args.push(Spanned::new(arg_string, arg_span));
        }

        (Spanned::new(cmd_name, cmd_span), args)
    }

    fn report_error(shell_error: ShellError, line: &str) {
        let report_err = |err: ShellError| {
            let report =
                Report::new(err).with_source_code(NamedSource::new("line", line.to_string()));
            eprintln!("{report:?}");
        };

        if let ShellError::Multiple(errors) = shell_error {
            for err in errors {
                report_err(err);
            }
        } else {
            report_err(shell_error);
        }
    }
}
