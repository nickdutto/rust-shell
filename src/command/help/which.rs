use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::command_registry::BUILTIN_COMMANDS;
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::syntax_shape::SyntaxShape;
use crate::system::env::get_env_paths;
use is_executable::is_executable;
use std::fmt::Write as FmtWrite;
use std::io::Write;

pub struct Which;

impl Command for Which {
    fn name(&self) -> &'static str {
        "which"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::Help)
            .required_positional("command_name", SyntaxShape::String, "Command name to find")
    }

    fn run(
        &self,
        call: Call,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let command_name = call.req::<String>(0)?;

        if command_name.is_empty() {
            writeln!(io_streams.error, "which: missing operand")?;
            return Ok(CommandData::ExitCode(ExitCode::SYNTAX_ERROR));
        }

        let mut buffer = String::new();

        if BUILTIN_COMMANDS.contains(&command_name.as_str()) {
            let _ = write!(buffer, "{command_name} is a shell system");
        } else {
            let Ok(paths) = get_env_paths("PATH") else {
                writeln!(
                    io_streams.error,
                    "which: error getting env paths for PATH variable"
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
                if *entry.file_name().unwrap_or_default() != *command_name {
                    continue;
                }

                buffer = format!("{} is {}", command_name, entry.to_str().unwrap_or_default());
                found = true;
                break;
            }

            if !found {
                buffer = format!("{command_name}: not found");
            }
        }

        writeln!(io_streams.output, "{}", buffer.trim())?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
