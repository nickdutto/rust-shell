use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::shell::shell_state::ShellState;
use crate::value::syntax_shape::SyntaxShape;
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

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::FileSystem)
            .required_positional("path", SyntaxShape::String, "The path to change to")
    }

    fn run(
        &self,
        call: Call,
        engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut final_exit_code = ExitCode::SUCCESS;

        let target = call.opt(0)?.unwrap_or(String::from("~"));

        let result = match target.trim() {
            "~" => {
                if let Some(home) = env::var_os("HOME") {
                    cd_set_dir(Path::new(&home), &engine_state.shell_state)
                } else {
                    Ok(())
                }
            }
            _ => cd_set_dir(Path::new(&target), &engine_state.shell_state),
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
