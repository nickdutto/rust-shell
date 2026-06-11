use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::shell::ShellState;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
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
                    match fs::read_to_string(Path::new(path)) {
                        Ok(content) => {
                            for line in content.lines() {
                                let line = line.trim();
                                if !line.is_empty() {
                                    let mut guard = shell_state.write().unwrap();
                                    guard.history.entries.push(line.to_string());
                                }
                            }
                        }
                        Err(err) => write_output(
                            format!("{}: error reading history file: {}", tokens.command, err)
                                .trim(),
                            OutputType::Stderr,
                            Some(&tokens),
                        ),
                    }
                }

                output_history = false;
            }
            "-w" => {
                if let Some(path) = arguments_iter.peek() {
                    save_history_file(&shell_state, path, WriteMode::Write, &tokens);
                }

                output_history = false;
            }
            "-a" => {
                if let Some(path) = arguments_iter.peek() {
                    save_history_file(&shell_state, path, WriteMode::Append, &tokens);
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
