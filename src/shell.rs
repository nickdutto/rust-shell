use crate::command::Command;
use std::error::Error;
use std::io::{stderr, stdout};

pub struct Shell;

impl Shell {
    pub fn start_session() -> Result<(), Box<dyn Error>> {
        loop {
            let mut rl = rustyline::DefaultEditor::new()?;

            let readline = rl.readline("$ ")?;
            if readline.trim().is_empty() {
                continue;
            }

            Command::parse_command(&readline).run_command(&mut stdout(), &mut stderr());
        }
    }
}
