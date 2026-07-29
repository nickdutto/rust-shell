use crate::config::Config;
use crate::engine::command::{BUILTIN_COMMANDS, Command, CommandData, CommandError, CommandType};
use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use crate::system::env::get_env_paths;
use is_executable::is_executable;
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct TypeCmd;

impl Command for TypeCmd {
    fn name(&self) -> &'static str {
        "type"
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
    ) -> Result<CommandData, CommandError> {
        let cmd_arg = args.first().map_or("", |s| s.item.as_str());
        if cmd_arg.is_empty() {
            writeln!(io_streams.error, "type: missing operand")?;
            return Ok(CommandData::ExitCode(ExitCode::SYNTAX_ERROR));
        }

        let mut buffer = String::new();

        if BUILTIN_COMMANDS.contains(&cmd_arg) {
            let _ = write!(buffer, "{cmd_arg} is a shell builtin");
        } else {
            let Ok(paths) = get_env_paths("PATH") else {
                writeln!(
                    io_streams.error,
                    "type: error getting env paths for PATH variable"
                )?;
                return Ok(CommandData::ExitCode(ExitCode::FAILURE));
            };

            let entries = paths
                .into_iter()
                .filter_map(|dir_path| dir_path.read_dir().ok())
                .flatten()
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter_map(|path| {
                    if is_executable(&path) {
                        return Some(path);
                    }

                    None
                });

            let mut found = false;
            for entry in entries {
                if *entry.file_name().unwrap_or_default() != *cmd_arg {
                    continue;
                }

                buffer = format!("{} is {}", cmd_arg, entry.to_str().unwrap_or_default());
                found = true;
                break;
            }

            if !found {
                buffer = format!("{cmd_arg}: not found");
            }
        }

        writeln!(io_streams.output, "{}", buffer.trim())?;
        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
