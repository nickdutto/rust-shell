use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::span::Spanned;
use crate::shell::aliases::Aliases;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Alias;

impl Command for Alias {
    fn name(&self) -> &'static str {
        "alias"
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
    ) -> Result<CommandData, ShellError> {
        let mut final_exit_code = ExitCode::SUCCESS;

        if args.is_empty() {
            let table = shell_state.read().unwrap().aliases.to_table();
            writeln!(io_streams.output, "{table}")?;
            return Ok(CommandData::ExitCode(final_exit_code));
        }

        let mut args_iter = args.iter();

        while let Some(arg) = args_iter.next() {
            match arg.item.as_str() {
                "-l" => {
                    let aliases = shell_state.read().unwrap().aliases.to_list_string();
                    writeln!(io_streams.output, "{}", aliases.trim_end())?;
                }

                "-t" => {
                    let table = shell_state.read().unwrap().aliases.to_table();
                    writeln!(io_streams.output, "{table}")?;
                }

                "-p" => {
                    if let Some(alias_key_arg) = args_iter.next() {
                        match shell_state
                            .write()
                            .unwrap()
                            .aliases
                            .get(&alias_key_arg.item)
                        {
                            Some(value) => {
                                writeln!(
                                    io_streams.output,
                                    "{}",
                                    Aliases::format_item_string(&alias_key_arg.item, value)
                                )?;
                            }
                            None => {
                                writeln!(
                                    io_streams.error,
                                    "alias: no alias named {} found",
                                    alias_key_arg.item
                                )?;
                            }
                        }
                    } else {
                        writeln!(io_streams.error, "alias: missing alias name after -p")?;
                        final_exit_code = ExitCode::SYNTAX_ERROR;
                    }
                }

                "-r" => {
                    if let Some(alias_key_arg) = args_iter.next() {
                        match shell_state
                            .write()
                            .unwrap()
                            .aliases
                            .remove(&alias_key_arg.item)
                        {
                            Some(value) => {
                                writeln!(
                                    io_streams.output,
                                    "alias: removed: {}",
                                    Aliases::format_item_string(&alias_key_arg.item, &value)
                                )?;
                            }
                            None => {
                                writeln!(
                                    io_streams.error,
                                    "alias: no alias named \"{}\" to remove",
                                    alias_key_arg.item
                                )?;
                                final_exit_code = ExitCode::FAILURE;
                            }
                        }
                    } else {
                        writeln!(io_streams.error, "alias: missing alias name after -r")?;
                        final_exit_code = ExitCode::SYNTAX_ERROR;
                    }
                }

                alias_name_arg => {
                    let eq_arg = args_iter.next().map(|s| s.item.as_str());
                    if eq_arg == Some("=") {
                        let mut aliased_args = vec![];
                        for arg in args_iter.by_ref() {
                            aliased_args.push(arg.clone());
                        }

                        if !aliased_args.is_empty() {
                            shell_state.write().unwrap().aliases.insert(
                                alias_name_arg.to_owned(),
                                aliased_args
                                    .iter()
                                    .map(|s| s.item.as_str())
                                    .collect::<Vec<&str>>()
                                    .join(" "),
                            );
                        }
                    } else {
                        writeln!(
                            io_streams.error,
                            "alias: missing = between alias name and aliased value. Example: alias ll = ls -la"
                        )?;
                        final_exit_code = ExitCode::SYNTAX_ERROR;
                    }
                }
            }
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}
