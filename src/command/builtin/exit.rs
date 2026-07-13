use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::shell::shell_state::ShellState;
use std::io::{Write, stdout};
use std::process;
use std::sync::{Arc, RwLock};

pub fn handle_exit(
    args: Vec<String>,
    shell_state: Arc<RwLock<ShellState>>,
    mut io_streams: IoStreams,
) -> std::io::Result<ExitCode> {
    if let Err(e) = shell_state
        .write()
        .unwrap()
        .history
        .exit_save_history_file()
    {
        writeln!(io_streams.error, "{}", e)?;
    }

    stdout().flush()?;

    let exit_code = args
        .first()
        .and_then(|c| c.parse::<i32>().ok())
        .unwrap_or(0);

    process::exit(exit_code);
}
