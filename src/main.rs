mod command;
mod env;
mod parser;
mod shell;

use crate::shell::Shell;

fn main() {
    Shell::start_session();
}
