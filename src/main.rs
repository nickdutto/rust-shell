mod command;
mod env;
mod shell;

use crate::shell::Shell;

fn main() {
    Shell::start_session();
}
