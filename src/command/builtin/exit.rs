use crate::shell::shell_state::ShellState;
use std::io::{Write, stdout};
use std::process;
use std::sync::{Arc, RwLock};

pub fn handle_exit(shell_state: Arc<RwLock<ShellState>>) {
    shell_state
        .write()
        .unwrap()
        .history
        .exit_save_history_file();
    stdout().flush().unwrap();
    process::exit(0);
}
