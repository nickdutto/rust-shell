use crate::engine::command::BUILTIN_COMMANDS;
use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::system::env::get_env_paths;
use is_executable::is_executable;
use std::fmt::Write as FmtWrite;
use std::io::Write;

pub fn handle_type(cmd: &str, mut io_streams: IoStreams) -> std::io::Result<ExitCode> {
    if cmd.is_empty() {
        writeln!(io_streams.error, "type: missing operand")?;
        return Ok(ExitCode::SYNTAX_ERROR);
    }

    let mut buffer = String::new();

    if BUILTIN_COMMANDS.contains(&cmd) {
        let _ = write!(buffer, "{} is a shell builtin", cmd);
    } else {
        let Ok(paths) = get_env_paths("PATH") else {
            writeln!(
                io_streams.error,
                "type: error getting env paths for PATH variable"
            )?;
            return Ok(ExitCode::FAILURE);
        };

        let entries = paths
            .into_iter()
            .filter_map(|dir_path| dir_path.read_dir().ok())
            .flatten()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter_map(|path| {
                if is_executable(&path) {
                    return Some(path);
                }

                None
            });

        let mut found = false;
        for entry in entries {
            if entry.file_name().unwrap_or_default() != cmd {
                continue;
            }

            buffer = format!("{} is {}", cmd, entry.to_str().unwrap_or_default());
            found = true;
            break;
        }

        if !found {
            buffer = format!("{}: not found", cmd);
        }
    }

    writeln!(io_streams.output, "{}", buffer.trim())?;
    Ok(ExitCode::SUCCESS)
}
