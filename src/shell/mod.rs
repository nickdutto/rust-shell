pub mod background_jobs;
pub mod completions;
pub mod config;
pub mod history;
pub mod prompt;
#[allow(clippy::module_inception)]
mod shell;
pub mod shell_helper;
pub mod shell_state;
pub mod suggestions;
pub mod variables;

pub use self::shell::*;
