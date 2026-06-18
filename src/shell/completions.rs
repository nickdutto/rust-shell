use rustyline::completion::Pair;
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
        self.specifications.insert(key.clone(), value.clone())
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.specifications.remove(key)
    }

    pub fn iter(&self) -> Iter<'_, String, String> {
        self.specifications.iter()
    }
}

impl Completions {
    pub fn complete_command(
        &self,
        partial_input: &str,
        pos: &mut usize,
        candidates: &mut Vec<Pair>,
        builtin_commands: &Vec<&'static str>,
        path_executables: &Arc<RwLock<Vec<String>>>,
    ) {
        for command in builtin_commands {
            if command.starts_with(partial_input) {
                candidates.push(Pair {
                    display: command.to_string(),
                    replacement: format!("{command} "),
                });
            }
        }

        for executable in path_executables.read().unwrap().iter() {
            if executable.starts_with(partial_input) {
                candidates.push(Pair {
                    display: executable.to_string(),
                    replacement: format!("{executable} "),
                });
            }
        }

        *pos = 0;
    }

    pub fn complete_filename(
        &self,
        partial_input: &str,
        pos: &mut usize,
        candidates: &mut Vec<Pair>,
    ) {
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

    pub fn complete_specification(
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

        if let Some(spec_script_path) = self.specifications.get(command) {
            if !partial_input.ends_with(' ') {
                if let Some(last_space) = partial_input.rfind(' ') {
                    *pos = last_space + 1;
                } else {
                    *pos = 0;
                }
            }

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
                        candidates.push(Pair {
                            display: trimmed.to_string(),
                            replacement: format!("{trimmed} "),
                        });
                    }
                }
            }
            return true;
        }

        false
    }
}
