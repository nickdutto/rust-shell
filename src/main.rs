pub mod command;
pub mod config;
pub mod engine;
pub mod io;
pub mod parser;
pub mod shell;
pub mod system;

use shell::Shell;

fn main() {
    Shell::new().start_session();
}
