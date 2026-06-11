use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::shell::ShellState;
use std::sync::{Arc, RwLock};

pub fn handle_history(tokens: Tokens, shell_state: Arc<RwLock<ShellState>>) {
    let guard = shell_state.read().unwrap();
    let history_len = guard.history.len();

    let limit = tokens
        .arguments
        .first()
        .and_then(|arg| arg.parse::<usize>().ok())
        .unwrap_or(history_len);

    let history: String = guard
        .history
        .iter()
        .enumerate()
        .skip(history_len.saturating_sub(limit))
        .map(|(idx, entry)| format!("{:>4}{}{:>2}{}\n", "", idx + 1, "", entry))
        .collect();

    write_output(history.trim_end(), OutputType::Stdout, &tokens);
}
