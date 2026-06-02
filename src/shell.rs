use crate::command::Command;
use std::io;
use std::io::{Write, stderr, stdout};

pub struct Shell;

impl Shell {
    pub fn start_session() {
        loop {
            print!("$ ");
            stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().is_empty() {
                continue;
            }

            Command::run_command(Command::parse_command(&input), &mut stdout(), &mut stderr());
        }
    }
}
