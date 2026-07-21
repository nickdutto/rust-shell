pub mod aliases;
pub mod background_jobs;
pub mod completer;
pub mod completions;
pub mod config;
pub mod highlighter;
pub mod history;
pub mod menus;
pub mod prompt;
#[allow(clippy::module_inception)]
mod shell;
pub mod shell_state;
pub mod suggestions;
pub mod variables;

pub use self::shell::*;
