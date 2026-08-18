use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::syntax_shape::SyntaxShape;
use crate::shell::aliases::Aliases;
use std::io::Write;

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

pub struct Alias;

impl Command for Alias {
    fn name(&self) -> &'static str {
        "alias"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        let mut signature = Signature::new(self.name()).category(Category::Shell);

        for spec in FORMAT_SPECS {
            signature = signature.switch(spec.name, spec.description, spec.short);
        }

        signature
            .switch("all", "list all aliases", Some('u'))
            .rest("add", SyntaxShape::String, "alias key and value to add")
            .named("remove", SyntaxShape::String, "alias to remove", Some('r'))
            .named(
                "print",
                SyntaxShape::String,
                "alias to get and print",
                Some('p'),
            )
    }

    fn run(
        &self,
        call: Call,
        engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut final_exit_code = ExitCode::SUCCESS;

        if !call.rest.is_empty() {
            let code = Self::add_alias(&call, engine_state, &mut io_streams)?;
            if code != ExitCode::SUCCESS {
                final_exit_code = code;
            }
        }

        if let Some(key) = call.opt_named::<String>("remove")? {
            let code = Self::remove_alias(&key, engine_state, &mut io_streams)?;
            if code != ExitCode::SUCCESS {
                final_exit_code = code;
            }
        }

        if let Some(key) = call.opt_named::<String>("print")? {
            let code = Self::print_alias(&key, engine_state, &mut io_streams)?;
            if code != ExitCode::SUCCESS {
                final_exit_code = code;
            }
        }

        if call.has_switch("all") {
            let code = Self::print_all_aliases(&call, engine_state, &mut io_streams)?;
            if code != ExitCode::SUCCESS {
                final_exit_code = code;
            }
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}

impl Alias {
    fn add_alias(
        args: &Call,
        engine_state: &EngineState,
        io_streams: &mut IoStreams,
    ) -> Result<ExitCode, ShellError> {
        let mut args_iter = args.rest.iter();

        let Some(key) = args_iter.next().map(|k| k.as_str()).transpose()? else {
            return Ok(ExitCode::SYNTAX_ERROR);
        };

        if args_iter.next().map(|a| a.as_str()).transpose()? != Some("=") {
            writeln!(
                io_streams.error,
                "alias: missing = between alias name and aliased value. Example: alias ll = 'ls -la'"
            )?;
            return Ok(ExitCode::SYNTAX_ERROR);
        }

        let mut aliased_args = vec![];
        for arg in args_iter {
            aliased_args.push(arg.as_str()?);
        }

        if !aliased_args.is_empty() {
            engine_state
                .shell_state
                .write()
                .unwrap()
                .aliases
                .insert(key.to_owned(), aliased_args.join(" "));
        }

        Ok(ExitCode::SUCCESS)
    }

    fn remove_alias(
        key: &str,
        engine_state: &EngineState,
        io_streams: &mut IoStreams,
    ) -> Result<ExitCode, ShellError> {
        if key.is_empty() {
            writeln!(io_streams.error, "alias: missing alias key after -r")?;
            return Ok(ExitCode::SYNTAX_ERROR);
        }

        match engine_state
            .shell_state
            .write()
            .unwrap()
            .aliases
            .remove(key)
        {
            Some(value) => {
                writeln!(
                    io_streams.output,
                    "alias: removed: {}",
                    Aliases::format_item_string(key, &value)
                )?;
            }
            None => {
                writeln!(io_streams.error, "alias: no alias key \"{key}\" to remove")?;
                return Ok(ExitCode::FAILURE);
            }
        }

        Ok(ExitCode::SUCCESS)
    }

    fn print_alias(
        key: &str,
        engine_state: &EngineState,
        io_streams: &mut IoStreams,
    ) -> Result<ExitCode, ShellError> {
        if key.is_empty() {
            writeln!(io_streams.error, "alias: missing alias key after -p")?;
            return Ok(ExitCode::SYNTAX_ERROR);
        }

        match engine_state.shell_state.read().unwrap().aliases.get(key) {
            Some(value) => {
                writeln!(
                    io_streams.output,
                    "{}",
                    Aliases::format_item_string(key, value)
                )?;
            }
            None => {
                writeln!(io_streams.error, "alias: no alias key {key} found")?;
            }
        }

        Ok(ExitCode::SUCCESS)
    }

    fn print_all_aliases(
        args: &Call,
        engine_state: &EngineState,
        io_streams: &mut IoStreams,
    ) -> Result<ExitCode, ShellError> {
        let mut format = Format::Table;
        for spec in FORMAT_SPECS {
            if args.has_switch(spec.name) {
                format = spec.mode.clone();
            }
        }

        let aliases = &engine_state.shell_state.read().unwrap().aliases;
        let output = match format {
            Format::List => aliases.to_list_string(),
            Format::Table => aliases.to_table().to_string(),
        };

        writeln!(io_streams.output, "{}", output.trim_end())?;

        Ok(ExitCode::SUCCESS)
    }
}
