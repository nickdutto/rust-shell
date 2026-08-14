use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::argument::ParsedArguments;
use crate::parser::span::Spanned;
use crate::parser::syntax_shape::SyntaxShape;
use crate::parser::value::Value;
use crate::shell::shell_state::ShellState;
use crate::shell::variables::{VariableError, Variables};
use std::io::Write;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
enum Format {
    List,
    Table,
}

struct NamedSpec<T> {
    name: &'static str,
    description: &'static str,
    short: Option<char>,
    mode: T,
}

const FORMAT_SPECS: &[NamedSpec<Format>] = &[
    NamedSpec {
        name: "list",
        description: "Print with list format",
        short: Some('l'),
        mode: Format::List,
    },
    NamedSpec {
        name: "table",
        description: "Print with table format",
        short: Some('t'),
        mode: Format::Table,
    },
];

pub struct Declare;

impl Command for Declare {
    fn name(&self) -> &'static str {
        "declare"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        let mut signature = Signature::new(self.name());

        for spec in FORMAT_SPECS {
            signature = signature.switch(spec.name, spec.description, spec.short);
        }

        signature
            .switch("all", "list all user defined variables", Some('u'))
            .rest("add", SyntaxShape::String, "variable key and value to add")
            .named(
                "remove",
                SyntaxShape::String,
                "variable to remove",
                Some('r'),
            )
            .named(
                "print",
                SyntaxShape::String,
                "variable to get and print",
                Some('p'),
            )
    }

    fn run(
        &self,
        _cmd: Spanned<String>,
        args: ParsedArguments,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut final_exit_code = ExitCode::SUCCESS;

        if let (Some(key), Some(value)) = (args.rest.first(), args.rest.get(2)) {
            let code = Self::add_variable(key, value, &shell_state, &mut io_streams)?;
            if code != ExitCode::SUCCESS {
                final_exit_code = code;
            }
        }

        if let Some(key) = args.opt_named::<String>("remove")? {
            let code = Self::remove_variable(&key, &shell_state, &mut io_streams)?;
            if code != ExitCode::SUCCESS {
                final_exit_code = code;
            }
        }

        if let Some(key) = args.opt_named::<String>("print")? {
            let code = Self::print_variable(&key, &shell_state, &mut io_streams)?;
            if code != ExitCode::SUCCESS {
                final_exit_code = code;
            }
        }

        if args.has_switch("all") {
            let code = Self::print_all_variables(&args, &shell_state, &mut io_streams)?;
            if code != ExitCode::SUCCESS {
                final_exit_code = code;
            }
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}

impl Declare {
    fn add_variable(
        key: &Value,
        value: &Value,
        shell_state: &Arc<RwLock<ShellState>>,
        io_streams: &mut IoStreams,
    ) -> Result<ExitCode, ShellError> {
        let key = key.as_str()?;
        let value = value.as_str()?;

        match shell_state
            .write()
            .unwrap()
            .variables
            .insert(key.to_owned(), value.to_owned())
        {
            Ok(_) => {}
            Err(VariableError::InvalidIdentifier { key, value }) => {
                writeln!(
                    io_streams.error,
                    "declare: `{}': not a valid identifier",
                    Variables::format_item_string(&key, &value)
                )?;
                return Ok(ExitCode::FAILURE);
            }
        }

        Ok(ExitCode::SUCCESS)
    }

    fn remove_variable(
        key: &str,
        shell_state: &Arc<RwLock<ShellState>>,
        io_streams: &mut IoStreams,
    ) -> Result<ExitCode, ShellError> {
        if key.is_empty() {
            writeln!(io_streams.error, "declare: missing variable key after -r")?;
            return Ok(ExitCode::SYNTAX_ERROR);
        }

        match shell_state.write().unwrap().variables.remove(key) {
            Some(value) => {
                writeln!(
                    io_streams.output,
                    "declare: removed: {}",
                    Variables::format_item_string(key, &value)
                )?;
            }
            None => {
                writeln!(io_streams.error, "declare: no variable key {key} to remove")?;
                return Ok(ExitCode::FAILURE);
            }
        }

        Ok(ExitCode::SUCCESS)
    }

    fn print_variable(
        key: &str,
        shell_state: &Arc<RwLock<ShellState>>,
        io_streams: &mut IoStreams,
    ) -> Result<ExitCode, ShellError> {
        if key.is_empty() {
            writeln!(io_streams.error, "declare: missing variable key after -p")?;
            return Ok(ExitCode::SYNTAX_ERROR);
        }

        match shell_state.read().unwrap().variables.get(key) {
            Ok(Some(value)) => {
                writeln!(
                    io_streams.output,
                    "{}",
                    Variables::format_item_string(key, value)
                )?;
            }
            Err(_) => {
                writeln!(io_streams.error, "declare: no variable key {key} found")?;
            }
            _ => {}
        }

        Ok(ExitCode::SUCCESS)
    }

    fn print_all_variables(
        args: &ParsedArguments,
        shell_state: &Arc<RwLock<ShellState>>,
        io_streams: &mut IoStreams,
    ) -> Result<ExitCode, ShellError> {
        let mut format = Format::Table;
        for spec in FORMAT_SPECS {
            if args.has_switch(spec.name) {
                format = spec.mode.clone();
            }
        }

        let variables = &shell_state.read().unwrap().variables;
        let output = match format {
            Format::List => variables.to_list_string(),
            Format::Table => variables.to_table().to_string(),
        };

        writeln!(io_streams.output, "{}", output.trim_end())?;

        Ok(ExitCode::SUCCESS)
    }
}
