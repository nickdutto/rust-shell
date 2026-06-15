use crate::io::stream::IoStreams;
use crate::shell::shell_state::ShellState;
use std::io::{Write, stdout};
use std::process;
use std::sync::{Arc, RwLock};

pub fn handle_exit(shell_state: Arc<RwLock<ShellState>>, mut io_streams: IoStreams) {
    match shell_state
        .write()
        .unwrap()
        .history
        .exit_save_history_file()
    {
        Ok(()) => (),
        Err(e) => writeln!(io_streams.error, "{}", e).unwrap(),
    }

    stdout().flush().unwrap();
    process::exit(0);
}
