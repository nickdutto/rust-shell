use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::env;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub struct Cd;

impl Command for Cd {
    fn name(&self) -> &'static str {
        "cd"
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

        let target = args.first().map_or("~", |s| s.item.as_str().trim());
        let result = match target {
            "~" => {
                if let Some(home) = env::var_os("HOME") {
                    cd_set_dir(Path::new(&home), &shell_state)
                } else {
                    Ok(())
                }
            }
            _ => cd_set_dir(Path::new(&target), &shell_state),
        };

        if let Err(e) = result {
            writeln!(io_streams.error, "{}", e.trim())?;
            final_exit_code = ExitCode::FAILURE;
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}

fn cd_set_dir(path: &Path, shell_state: &Arc<RwLock<ShellState>>) -> Result<(), String> {
    match env::set_current_dir(path) {
        Ok(()) => {
            shell_state.write().unwrap().current_directory = env::current_dir().unwrap_or_default();
            Ok(())
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Err(format!(
            "cd: {}: No such file or directory",
            path.to_str().unwrap_or_default()
        )),
        Err(e) => Err(format!("cd: {}: {}", path.display(), e)),
    }
}
