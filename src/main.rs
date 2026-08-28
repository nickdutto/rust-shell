pub mod command;
pub mod config;
pub mod engine;
pub mod error;
pub mod format;
pub mod io;
pub mod network;
pub mod parser;
pub mod shell;
pub mod system;
pub mod value;

use shell::Shell;

fn main() {
    Shell::start_session();
}
