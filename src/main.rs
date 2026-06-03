mod command;
mod env;
mod parser;
mod shell;
mod writer;

use crate::shell::Shell;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Shell::start_session()?;

    Ok(())
}
