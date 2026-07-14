use reedline::{Span, Suggestion};
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::{env, fs, process};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompletionError {
    #[error("complete: {key}: no completion specification")]
    MissingSpecification { key: String },
}

#[derive(Default)]
pub struct Completions {
    specifications: HashMap<String, String>,
}

struct CompletionPath {
    path: String,
    is_dir: bool,
}

impl Completions {
    pub fn new() -> Self {
        Self {
            specifications: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.specifications.get(key)
    }

    pub fn get_key_value(&self, key: &str) -> Result<(&String, &String), CompletionError> {
        if let Some(script) = self.specifications.get_key_value(key) {
            Ok(script)
        } else {
            Err(CompletionError::MissingSpecification {
                key: key.to_string(),
            })
        }
    }

    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        self.specifications.insert(key, value)
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.specifications.remove(key)
    }

    pub fn iter(&self) -> Iter<'_, String, String> {
        self.specifications.iter()
    }

    pub fn complete_command(
        &self,
        partial_input: &str,
        pos: usize,
        suggestions: &mut Vec<Suggestion>,
        builtin_commands: &'static [&'static str],
        path_executables: &Arc<RwLock<Vec<String>>>,
    ) {
        let span = Span::new(0, pos);

        for command in builtin_commands {
            if command.starts_with(partial_input) {
                suggestions.push(Suggestion {
                    value: format!("{command} "),
                    display_override: Some(command.to_string()),
                    description: Some("Builtin command".to_string()),
                    style: None,
                    extra: None,
                    match_indices: None,
                    span,
                    append_whitespace: false,
                });
            }
        }

        for executable in path_executables.read().unwrap().iter() {
            if executable.starts_with(partial_input) {
                suggestions.push(Suggestion {
                    value: format!("{executable} "),
                    display_override: Some(executable.clone()),
                    description: Some("Path executable".to_string()),
                    style: None,
                    extra: None,
                    match_indices: None,
                    span,
                    append_whitespace: false,
                });
            }
        }
    }

    pub fn complete_filename(
        &self,
        partial_input: &str,
        pos: usize,
        suggestions: &mut Vec<Suggestion>,
    ) {
        let last_partial_input = match partial_input.rfind(' ') {
            Some(idx) => &partial_input[idx + 1..],
            None => partial_input,
        };

        let start_pos = partial_input.rfind(' ').map_or(0, |idx| idx + 1);
        let span = Span::new(start_pos, pos);

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
        }

        let dir_path = if !path.is_empty() {
            PathBuf::from(&path)
        } else {
            env::current_dir().unwrap_or(PathBuf::from("."))
        };

        let mut completion_paths: Vec<CompletionPath> = fs::read_dir(dir_path)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
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

                suggestions.push(Suggestion {
                    value: path_pair.clone(),
                    display_override: Some(path_pair.trim().to_string()),
                    description: None,
                    style: None,
                    extra: None,
                    match_indices: None,
                    span,
                    append_whitespace: false,
                });
            }
        }
    }

    pub fn complete_specification(
        &self,
        line: &str,
        partial_input: &str,
        pos: usize,
        suggestions: &mut Vec<Suggestion>,
    ) -> bool {
        let words: Vec<&str> = partial_input.split_whitespace().collect();
        if words.is_empty() {
            return false;
        }

        let command = words[0];
        let (current_word, preceding_word) = if partial_input.ends_with(' ') {
            ("", words.last().copied().unwrap_or(""))
        } else {
            (
                words.last().copied().unwrap_or(""),
                if words.len() >= 2 {
                    words[words.len() - 2]
                } else {
                    ""
                },
            )
        };

        if let Some(spec_script_path) = self.specifications.get(command) {
            let start_pos = if !partial_input.ends_with(' ') {
                partial_input
                    .rfind(' ')
                    .map_or(0, |last_space| last_space + 1)
            } else {
                pos
            };
            let span = Span::new(start_pos, pos);

            if let Ok(output) = process::Command::new(spec_script_path)
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
                        suggestions.push(Suggestion {
                            value: format!("{trimmed} "),
                            display_override: Some(trimmed.to_string()),
                            description: Some("Custom spec completion".to_string()),
                            style: None,
                            extra: None,
                            match_indices: None,
                            span,
                            append_whitespace: false,
                        });
                    }
                }
            }
            return true;
        }

        false
    }
}

impl<'a> IntoIterator for &'a Completions {
    type Item = (&'a String, &'a String);
    type IntoIter = Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
