use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::shell::history::WriteMode;
use crate::shell::shell_state::ShellState;
use std::sync::{Arc, RwLock};

pub fn handle_history(tokens: Tokens, shell_state: Arc<RwLock<ShellState>>) {
    let history_len = shell_state.read().unwrap().history.entries.len();
    let mut limit: usize = history_len;
    let mut output_history = true;

    let mut arguments_iter = tokens.arguments.iter().peekable();
    while let Some(argument) = arguments_iter.next() {
        match argument.as_str() {
            "-r" => {
                if let Some(path) = arguments_iter.peek() {
                    shell_state.write().unwrap().history.read_history_file(path);
                }

                output_history = false;
            }
            "-w" => {
                if let Some(path) = arguments_iter.peek() {
                    shell_state
                        .write()
                        .unwrap()
                        .history
                        .save_history_file(path, WriteMode::Write);
                }

                output_history = false;
            }
            "-a" => {
                if let Some(path) = arguments_iter.peek() {
                    shell_state
                        .write()
                        .unwrap()
                        .history
                        .save_history_file(path, WriteMode::Append);
                }

                output_history = false;
            }
            _ => {
                limit = tokens
                    .arguments
                    .first()
                    .and_then(|arg| arg.parse::<usize>().ok())
                    .unwrap_or(history_len);
            }
        }
    }

    if output_history {
        let output: String = shell_state
            .read()
            .unwrap()
            .history
            .entries
            .iter()
            .enumerate()
            .skip(history_len.saturating_sub(limit))
            .map(|(idx, entry)| format!("{:>4}{}{:>2}{}\n", "", idx + 1, "", entry))
            .collect();

        write_output(output.trim_end(), OutputType::Stdout, Some(&tokens));
    }
}
