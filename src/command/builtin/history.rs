use crate::engine::command::{Command, CommandData, CommandError, CommandType};
use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::shell::config::Config;
use crate::shell::history::WriteMode;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct History;

impl Command for History {
    fn name(&self) -> &str {
        "history"
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

        let history_len = shell_state.read().unwrap().history.entries.len();
        let mut limit: usize = history_len;
        let mut output_history = true;
        let mut args_iter = args.iter().peekable();

        while let Some(arg) = args_iter.next() {
            match arg.as_str() {
                flag @ ("-r" | "-w" | "-a") => {
                    output_history = false;

                    if let Some(path) = args_iter.peek() {
                        let mut guard = shell_state.write().unwrap();

                        let result = match flag {
                            "-r" => guard.history.read_history_file(path),
                            "-w" => guard.history.save_history_file(path, WriteMode::Write),
                            "-a" => guard.history.save_history_file(path, WriteMode::Append),
                            _ => unreachable!(),
                        };

                        if let Err(e) = result {
                            writeln!(io_streams.error, "{}", e)?;
                            final_exit_code = ExitCode::FAILURE;
                        }
                    }
                }
                _ => {
                    limit = args
                        .first()
                        .and_then(|arg| arg.parse::<usize>().ok())
                        .unwrap_or(history_len);
                }
            }
        }

        if output_history {
            let output: String = shell_state
                .read()
                .unwrap()
                .history
                .entries
                .iter()
                .enumerate()
                .skip(history_len.saturating_sub(limit))
                .map(|(idx, entry)| format!("{:>4}{}{:>2}{}\n", "", idx + 1, "", entry))
                .collect();

            writeln!(io_streams.output, "{}", output.trim_end())?;
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}
