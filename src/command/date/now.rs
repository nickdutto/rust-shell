use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use jiff::Zoned;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Now;

impl Command for Now {
    fn name(&self) -> &'static str {
        "now"
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
        _shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut final_exit_code = ExitCode::SUCCESS;
        let mut args_iter = args.iter();
        let now = Zoned::now();

        while let Some(arg) = args_iter.next() {
            match arg.item.as_str() {
                "-d" => {
                    writeln!(io_streams.output, "{}", now.strftime("%d"))?;
                }
                "-m" => {
                    writeln!(io_streams.output, "{}", now.strftime("%m"))?;
                }
                "-y" => {
                    writeln!(io_streams.output, "{}", now.strftime("%Y"))?;
                }
                "-hh" => {
                    writeln!(io_streams.output, "{}", now.strftime("%H"))?;
                }
                "-mm" => {
                    writeln!(io_streams.output, "{}", now.strftime("%M"))?;
                }
                "-ss" => {
                    writeln!(io_streams.output, "{}", now.strftime("%S"))?;
                }
                "-o" => {
                    writeln!(io_streams.output, "{}", now.strftime("%:z"))?;
                }
                "-q" => {
                    writeln!(io_streams.output, "{}", now.strftime("%:Q"))?;
                }
                "-tz" => {
                    writeln!(io_streams.output, "{}", now.strftime("%:Z"))?;
                }
                "-a" => {
                    writeln!(io_streams.output, "{}", now.strftime("%A"))?;
                }
                "-b" => {
                    writeln!(io_streams.output, "{}", now.strftime("%B"))?;
                }
                "-raw" => {
                    writeln!(io_streams.output, "{now}")?;
                }
                "-iso" => {
                    writeln!(
                        io_streams.output,
                        "{}",
                        now.strftime("%Y-%m-%dT%H:%M:%S%:z")
                    )?;
                }
                "-date" => {
                    writeln!(io_streams.output, "{}", now.strftime("%d/%m/%y"))?;
                }
                "-time" => {
                    writeln!(io_streams.output, "{}", now.strftime("%T"))?;
                }
                "-f" => {
                    if let Some(format_arg) = args_iter.next() {
                        match jiff::fmt::strtime::format(&format_arg.item, &now) {
                            Ok(f) => writeln!(io_streams.output, "{f}")?,
                            Err(e) => {
                                writeln!(io_streams.error, "now: {e}")?;
                                final_exit_code = ExitCode::FAILURE;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if args.is_empty() {
            writeln!(io_streams.output, "{now}")?;
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}
