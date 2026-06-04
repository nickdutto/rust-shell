use crate::command::BUILTIN_COMMANDS;
use crate::env::get_env_path_executables;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::env;

pub struct ShellHelper<'a> {
    builtin_commands: Vec<&'a str>,
    path_executables: Vec<String>,
}

impl<'a> ShellHelper<'a> {
    pub fn new() -> Self {
        ShellHelper {
            builtin_commands: BUILTIN_COMMANDS.to_vec(),
            path_executables: get_env_path_executables("PATH"),
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
            self.complete_filename(line, &mut pos, &mut candidates);
        }

        candidates.sort_by(|a, b| a.replacement.cmp(&b.replacement));
        candidates.dedup_by(|a, b| a.replacement == b.replacement);

        Ok((pos, candidates))
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

        for executable in &self.path_executables {
            if executable.starts_with(partial_input) {
                candidates.push(Pair {
                    display: executable.to_string(),
                    replacement: format!("{} ", executable),
                });
            }
        }

        *pos = 0;
    }

    fn complete_filename(&self, line: &str, pos: &mut usize, candidates: &mut Vec<Pair>) {
        let last_partial_input = match line.split(' ').next_back() {
            Some(last) => last.to_lowercase(),
            None => return,
        };

        if let Some(last_whitespace) = line.rfind(' ').map(|idx| idx + 1) {
            *pos = last_whitespace;
        }

        let filenames: Vec<String> = env::current_dir()
            .into_iter()
            .flat_map(|dir| dir.read_dir())
            .flatten()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_file()))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();

        for filename in filenames {
            if filename.to_lowercase().starts_with(&last_partial_input) {
                candidates.push(Pair {
                    display: filename.to_string(),
                    replacement: format!("{} ", filename),
                });
            }
        }
    }
}
