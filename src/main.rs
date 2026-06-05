pub mod command;
pub mod io;
pub mod shell;
pub mod system;

use shell::Shell;

fn main() {
    Shell::start_session();
}
