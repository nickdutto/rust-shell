use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::argument::ParsedArguments;
use crate::parser::span::Spanned;
use crate::parser::syntax_shape::SyntaxShape;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Complete;

impl Command for Complete {
    fn name(&self) -> &'static str {
        "complete"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .rest(
                "add",
                SyntaxShape::String,
                "completion script key and path to add",
            )
            .named(
                "remove",
                SyntaxShape::String,
                "Remove completion",
                Some('r'),
            )
            .named(
                "print",
                SyntaxShape::String,
                "completion script to get and print",
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

        if !args.rest.is_empty() {
            let code = Self::add_completion(&args, &shell_state, &mut io_streams)?;
            if code != ExitCode::SUCCESS {
                final_exit_code = code;
            }
        }

        if let Some(key) = args.opt_named::<String>("remove")? {
            Self::remove_completion(&key, &shell_state);
        }

        if let Some(key) = args.opt_named::<String>("print")? {
            let code = Self::print_completion(&key, &shell_state, &mut io_streams)?;
            if code != ExitCode::SUCCESS {
                final_exit_code = code;
            }
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}

impl Complete {
    fn add_completion(
        args: &ParsedArguments,
        shell_state: &Arc<RwLock<ShellState>>,
        io_streams: &mut IoStreams,
    ) -> Result<ExitCode, ShellError> {
        let mut args_iter = args.rest.iter();

        let Some(key) = args_iter.next().map(|k| k.as_str()).transpose()? else {
            return Ok(ExitCode::SYNTAX_ERROR);
        };

        let Some(path) = args_iter.next().map(|k| k.as_str()).transpose()? else {
            writeln!(
                io_streams.error,
                "complete: missing script path after key. Example: complete script_name /path"
            )?;
            return Ok(ExitCode::SYNTAX_ERROR);
        };

        shell_state
            .write()
            .unwrap()
            .completions
            .insert(key.to_owned(), path.to_owned());

        Ok(ExitCode::SUCCESS)
    }
    fn remove_completion(key: &str, shell_state: &Arc<RwLock<ShellState>>) {
        shell_state.write().unwrap().completions.remove(key);
    }

    fn print_completion(
        key: &str,
        shell_state: &Arc<RwLock<ShellState>>,
        io_streams: &mut IoStreams,
    ) -> Result<ExitCode, ShellError> {
        if key.is_empty() {
            writeln!(
                io_streams.error,
                "complete: missing specification name for -p",
            )?;
            return Ok(ExitCode::SYNTAX_ERROR);
        }

        match shell_state.read().unwrap().completions.get_key_value(key) {
            Ok((name, path)) => {
                writeln!(io_streams.output, "complete -C '{path}' {name}")?;
            }
            Err(e) => {
                writeln!(io_streams.error, "{e}")?;
                return Ok(ExitCode::FAILURE);
            }
        }

        Ok(ExitCode::SUCCESS)
    }
}
