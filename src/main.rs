mod command;
mod env;
mod parser;
mod shell;
mod shell_helper;
mod writer;

use crate::shell::Shell;

fn main() {
    Shell::start_session();
}
