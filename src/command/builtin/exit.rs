use std::io::{Write, stdout};
use std::process;

pub fn handle_exit() {
    stdout().flush().unwrap();
    process::exit(0);
}
