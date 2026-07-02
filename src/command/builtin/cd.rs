use crate::io::stream::IoStreams;
use crate::parser::CommandNode;
use crate::shell::shell_state::ShellState;
use std::env;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub fn handle_cd(
    tokens: CommandNode,
    shell_state: Arc<RwLock<ShellState>>,
    mut io_streams: IoStreams,
) {
    let target = tokens
        .arguments
        .first()
        .map(|s| s.as_str().trim())
        .unwrap_or("~");

    let result = match target {
        "~" => {
            if let Some(home) = env::var_os("HOME") {
                cd_set_dir(Path::new(&home), shell_state)
            } else {
                Ok(())
            }
        }
        _ => cd_set_dir(Path::new(&target), shell_state),
    };

    if let Err(e) = result {
        writeln!(io_streams.error, "{}", e.trim()).unwrap();
    }
}

fn cd_set_dir(path: &Path, shell_state: Arc<RwLock<ShellState>>) -> Result<(), String> {
    match env::set_current_dir(path) {
        Ok(_) => {
            shell_state.write().unwrap().current_directory = env::current_dir().unwrap_or_default();
            Ok(())
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Err(format!(
            "cd: {}: No such file or directory",
            path.to_str().unwrap()
        )),
        Err(e) => Err(format!("cd: {}: {}", path.display(), e)),
    }
}
