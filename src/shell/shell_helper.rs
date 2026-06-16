use crate::command::BUILTIN_COMMANDS;
use crate::shell::shell_state::ShellState;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::{env, fs, process};

struct CompletionPath {
    path: String,
    is_dir: bool,
}

pub struct ShellHelper<'a> {
    shell_state: Arc<RwLock<ShellState>>,
    builtin_commands: Vec<&'a str>,
    path_executables: Arc<RwLock<Vec<String>>>,
}

impl<'a> ShellHelper<'a> {
    pub fn new(
        path_executables: Arc<RwLock<Vec<String>>>,
        shell_state: Arc<RwLock<ShellState>>,
    ) -> Self {
        ShellHelper {
            builtin_commands: BUILTIN_COMMANDS.to_vec(),
            path_executables,
            shell_state,
        }
    }
}

impl<'a> Helper for ShellHelper<'a> {}
impl<'a> Hinter for ShellHelper<'a> {
    type Hint = String;
}
impl<'a> Highlighter for ShellHelper<'a> {}
impl<'a> Validator for ShellHelper<'a> {}
impl<'a> Completer for ShellHelper<'a> {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let mut pos = pos;
        let mut candidates = Vec::new();

        let partial_input = &line[..pos];
        if partial_input.is_empty() {
            return Ok((pos, candidates));
        }

        if !partial_input.contains(' ') {
            self.complete_command(partial_input, &mut pos, &mut candidates);
        } else {
            let specification_found =
                self.complete_specification_script(line, partial_input, &mut pos, &mut candidates);
            if !specification_found {
                self.complete_filename(partial_input, &mut pos, &mut candidates);
            }
        }

        candidates.sort_by(|a, b| a.replacement.cmp(&b.replacement));
        candidates.dedup_by(|a, b| a.replacement == b.replacement);

        Ok((pos, candidates))
    }
}

impl<'a> ShellHelper<'a> {
    fn complete_specification_script(
        &self,
        line: &str,
        partial_input: &str,
        pos: &mut usize,
        candidates: &mut Vec<Pair>,
    ) -> bool {
        let words: Vec<&str> = partial_input.split_whitespace().collect();
        if words.is_empty() {
            return false;
        }

        let command = words[0];
        let (current_word, preceding_word) = if partial_input.ends_with(' ') {
            ("", words.last().cloned().unwrap_or(""))
        } else {
            (
                words.last().cloned().unwrap_or(""),
                if words.len() >= 2 {
                    words[words.len() - 2]
                } else {
                    ""
                },
            )
        };

        if let Ok(shell_state_guard) = self.shell_state.read()
            && let Some(value) = shell_state_guard.completions.get(command)
        {
            if !partial_input.ends_with(' ') {
                if let Some(last_space) = partial_input.rfind(' ') {
                    *pos = last_space + 1;
                } else {
                    *pos = 0;
                }
            }

            if let Ok(output) = process::Command::new(value)
                .args([command, current_word, preceding_word])
                .env("COMP_LINE", line)
                .env("COMP_POINT", partial_input.len().to_string())
                .output()
                && output.status.success()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        candidates.push(Pair {
                            display: trimmed.to_string(),
                            replacement: format!("{} ", trimmed),
                        });
                    }
                }
            }
            return true;
        }

        false
    }
}

impl<'a> ShellHelper<'a> {
    fn complete_command(&self, partial_input: &str, pos: &mut usize, candidates: &mut Vec<Pair>) {
        for command in &self.builtin_commands {
            if command.starts_with(partial_input) {
                candidates.push(Pair {
                    display: command.to_string(),
                    replacement: format!("{} ", command),
                });
            }
        }

        if let Ok(executables) = self.path_executables.read() {
            for executable in executables.iter() {
                if executable.starts_with(partial_input) {
                    candidates.push(Pair {
                        display: executable.to_string(),
                        replacement: format!("{} ", executable),
                    });
                }
            }
        }

        *pos = 0;
    }

    fn complete_filename(&self, partial_input: &str, pos: &mut usize, candidates: &mut Vec<Pair>) {
        let last_partial_input = match partial_input.rfind(' ') {
            Some(idx) => &partial_input[idx + 1..],
            None => partial_input,
        };

        if let Some(last_whitespace) = partial_input.rfind(' ').map(|idx| idx + 1) {
            *pos = last_whitespace;
        }

        let path;
        let partial_filename;

        let char = '/';
        if let Some(idx) = last_partial_input.rfind(char) {
            let cut_index = idx + char.len_utf8();
            path = last_partial_input[..cut_index].to_string();
            partial_filename = last_partial_input[cut_index..].to_string();
        } else {
            path = String::new();
            partial_filename = last_partial_input.to_string();
        };

        let dir_path = if !path.is_empty() {
            PathBuf::from(&path)
        } else {
            env::current_dir().ok().unwrap().to_path_buf()
        };

        let mut completion_paths: Vec<CompletionPath> = fs::read_dir(dir_path)
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let file_name = entry.file_name().into_string().ok()?;
                let file_type = entry.file_type().ok()?;

                Some(CompletionPath {
                    path: file_name,
                    is_dir: file_type.is_dir(),
                })
            })
            .collect();

        completion_paths.sort_by(|a, b| a.path.cmp(&b.path));

        for completion_path in completion_paths {
            if last_partial_input.is_empty()
                || last_partial_input.ends_with('/')
                || completion_path.path.starts_with(&partial_filename)
            {
                let path_pair = if completion_path.is_dir {
                    format!("{}{}/", path, completion_path.path)
                } else {
                    format!("{}{} ", path, completion_path.path)
                };

                candidates.push(Pair {
                    display: path_pair.trim().to_string(),
                    replacement: path_pair,
                });
            }
        }
    }
}
