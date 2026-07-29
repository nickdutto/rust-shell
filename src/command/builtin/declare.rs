use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandError, CommandType};
use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use crate::shell::variables::{VariableError, Variables};
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Declare;

impl Command for Declare {
    fn name(&self) -> &'static str {
        "declare"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn run(
        &self,
        _cmd: Spanned<String>,
        args: Vec<Spanned<String>>,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, CommandError> {
        let mut final_exit_code = ExitCode::SUCCESS;

        if args.is_empty() {
            let table = shell_state.read().unwrap().variables.to_table();
            writeln!(io_streams.output, "{table}")?;
            return Ok(CommandData::ExitCode(final_exit_code));
        }

        let mut args_iter = args.iter();

        while let Some(arg) = args_iter.next() {
            match arg.item.as_str() {
                "-l" => {
                    let variables = shell_state.read().unwrap().variables.to_list_string();
                    writeln!(io_streams.output, "{}", variables.trim_end())?;
                }

                "-t" => {
                    let table = shell_state.read().unwrap().variables.to_table();
                    writeln!(io_streams.output, "{table}")?;
                }

                "-p" => {
                    if let Some(name_arg) = args_iter.next() {
                        match shell_state.write().unwrap().variables.get(&name_arg.item) {
                            Ok(Some(value)) => {
                                writeln!(
                                    io_streams.output,
                                    "{}",
                                    Variables::format_item_string(&name_arg.item, value)
                                )?;
                            }
                            Err(_) => {
                                writeln!(
                                    io_streams.error,
                                    "declare: no variable named {} found",
                                    name_arg.item
                                )?;
                            }
                            _ => {}
                        }
                    } else {
                        writeln!(io_streams.error, "declare: missing variable name after -p")?;
                        final_exit_code = ExitCode::SYNTAX_ERROR;
                    }
                }

                "-r" => {
                    if let Some(name_arg) = args_iter.next() {
                        match shell_state
                            .write()
                            .unwrap()
                            .variables
                            .remove(&name_arg.item)
                        {
                            Some(value) => {
                                writeln!(
                                    io_streams.output,
                                    "declare: removed: {}",
                                    Variables::format_item_string(&name_arg.item, &value)
                                )?;
                            }
                            None => {
                                writeln!(
                                    io_streams.error,
                                    "declare: no variable named {} to remove",
                                    name_arg.item
                                )?;
                                final_exit_code = ExitCode::FAILURE;
                            }
                        }
                    } else {
                        writeln!(io_streams.error, "declare: missing variable name after -r")?;
                        final_exit_code = ExitCode::SYNTAX_ERROR;
                    }
                }

                name_arg => {
                    if args_iter.next().map(|s| s.item.as_str()) != Some("=") {
                        writeln!(
                            io_streams.error,
                            "declare: missing = between variable name and value. Example: declare TARGET = ./src"
                        )?;
                        final_exit_code = ExitCode::SYNTAX_ERROR;
                        continue;
                    }

                    if let Some(value_arg) = args_iter.next() {
                        match shell_state
                            .write()
                            .unwrap()
                            .variables
                            .insert(name_arg.to_owned(), value_arg.item.clone())
                        {
                            Ok(_) => {}
                            Err(VariableError::InvalidIdentifier { key, value }) => {
                                writeln!(
                                    io_streams.error,
                                    "declare: `{}': not a valid identifier",
                                    Variables::format_item_string(&key, &value)
                                )?;
                                final_exit_code = ExitCode::FAILURE;
                            }
                        }
                    }
                }
            }
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}
