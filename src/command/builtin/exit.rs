use crate::io::writer::{OutputType, write_output};
use crate::shell::shell_state::ShellState;
use std::io::{Write, stdout};
use std::process;
use std::sync::{Arc, RwLock};

pub fn handle_exit(shell_state: Arc<RwLock<ShellState>>) {
    match shell_state
        .write()
        .unwrap()
        .history
        .exit_save_history_file()
    {
        Ok(()) => (),
        Err(err) => write_output(&err.to_string(), OutputType::Stderr, None),
    }

    stdout().flush().unwrap();
    process::exit(0);
}
