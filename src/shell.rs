use crate::command::Command;
use std::io;
use std::io::Write;

pub struct Shell;

impl Shell {
    pub fn new() -> Self {
        Self
    }

    pub fn start_session() {
        loop {
            print!("$ ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().is_empty() {
                continue;
            }

            let command = Command::parse_command(&input);
            Command::run_command(command);
        }
    }
}
