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
            flag @ ("-r" | "-w" | "-a") => {
                output_history = false;

                if let Some(path) = arguments_iter.peek() {
                    let mut guard = shell_state.write().unwrap();

                    let result = match flag {
                        "-r" => guard.history.read_history_file(path),
                        "-w" => guard.history.save_history_file(path, WriteMode::Write),
                        "-a" => guard.history.save_history_file(path, WriteMode::Append),
                        _ => unreachable!(),
                    };

                    if let Err(err) = result {
                        write_output(&err.to_string(), OutputType::Stderr, Some(&tokens));
                    }
                }
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
