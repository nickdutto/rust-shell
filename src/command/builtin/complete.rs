use crate::engine::command::{Command, CommandData, CommandError, CommandType};
use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::shell::config::Config;
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

    fn run(
        &self,
        _cmd: &str,
        args: Vec<String>,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, CommandError> {
        let mut final_exit_code = ExitCode::SUCCESS;
        let mut args_iter = args.iter();

        while let Some(arg) = args_iter.next() {
            match arg.as_str() {
                "-C" => {
                    if let Some(path_arg) = args_iter.next()
                        && let Some(name_arg) = args_iter.next()
                    {
                        shell_state
                            .write()
                            .unwrap()
                            .completions
                            .insert(name_arg.clone(), path_arg.clone());
                    }
                }
                "-r" => {
                    if let Some(name_arg) = args_iter.next() {
                        shell_state.write().unwrap().completions.remove(name_arg);
                    }
                }
                "-p" => {
                    if let Some(name_arg) = args_iter.next() {
                        match shell_state
                            .read()
                            .unwrap()
                            .completions
                            .get_key_value(name_arg)
                        {
                            Ok((name, path)) => {
                                writeln!(io_streams.output, "complete -C '{path}' {name}")?;
                            }
                            Err(e) => {
                                writeln!(io_streams.error, "{e}")?;
                                final_exit_code = ExitCode::FAILURE;
                            }
                        }
                    } else {
                        writeln!(
                            io_streams.error,
                            "complete: missing specification name for -p",
                        )?;
                        final_exit_code = ExitCode::SYNTAX_ERROR;
                    }
                }
                _ => (),
            }
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}
