use crate::io::writer::{OutputType, write_output};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::{env, fs};

#[derive(PartialEq)]
pub enum WriteMode {
    Write,
    Append,
}

pub struct History {
    pub entries: Vec<String>,
    pub append_index: usize,
}

impl Default for History {
    fn default() -> Self {
        let mut history = Self {
            entries: vec![],
            append_index: 0,
        };

        history.startup_history_file();

        history
    }
}

impl History {
    pub fn startup_history_file(&mut self) {
        if let Some(history_file_path) = env::var_os("HISTFILE") {
            self.read_history_file(history_file_path.to_str().unwrap())
        }
    }

    pub fn exit_save_history_file(&mut self) {
        if let Some(history_file_path) = env::var_os("HISTFILE") {
            self.save_history_file(history_file_path.to_str().unwrap(), WriteMode::Append);
        }
    }

    pub fn read_history_file(&mut self, path: &str) {
        match fs::read_to_string(Path::new(path)) {
            Ok(content) => {
                self.entries.append(
                    &mut content
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<String>>(),
                );
                self.append_index = self.entries.len();
            }
            Err(err) => write_output(
                format!("history: error reading history file: {}", err).trim(),
                OutputType::Stderr,
                None,
            ),
        }
    }

    pub fn save_history_file(&mut self, path: &str, mode: WriteMode) {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).ok();
        }

        let skip = if mode == WriteMode::Append {
            self.append_index
        } else {
            0
        };

        let result = OpenOptions::new()
            .create(true)
            .write(mode == WriteMode::Write)
            .append(mode == WriteMode::Append)
            .open(path)
            .and_then(|mut file| {
                let output: String = self
                    .entries
                    .iter()
                    .skip(skip)
                    .map(|entry| format!("{}\n", entry))
                    .collect();

                file.write_all(output.as_bytes())
            });

        match result {
            Ok(_) => {
                if mode == WriteMode::Append {
                    self.append_index = self.entries.len()
                }
            }
            Err(err) => write_output(
                format!("history: error writing history file: {}", err).trim(),
                OutputType::Stderr,
                None,
            ),
        }
    }
}
