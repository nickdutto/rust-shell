use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{Error, Write};
use std::path::Path;
use std::{env, fs};
use thiserror::Error;

#[derive(PartialEq)]
pub enum WriteMode {
    Write,
    Append,
}

#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("history: error reading history file: {error}")]
    FileReadError { error: Error },
    #[error("history: error writing history file: {error}")]
    FileWriteError { error: Error },
}

#[derive(Default)]
pub struct History {
    pub entries: Vec<String>,
    pub append_index: usize,
}

impl History {
    pub fn new() -> Self {
        let mut history = Self {
            entries: vec![],
            append_index: 0,
        };

        match history.startup_history_file() {
            Ok(()) => {}
            Err(e) => eprintln!("{}", e),
        }

        history
    }

    pub fn startup_history_file(&mut self) -> Result<(), HistoryError> {
        if let Some(history_file_path) = env::var_os("HISTFILE") {
            self.read_history_file(history_file_path.to_str().unwrap())?
        }

        Ok(())
    }

    pub fn exit_save_history_file(&mut self) -> Result<(), HistoryError> {
        if let Some(history_file_path) = env::var_os("HISTFILE") {
            self.save_history_file(history_file_path.to_str().unwrap(), WriteMode::Append)?
        }

        Ok(())
    }

    pub fn read_history_file(&mut self, path: &str) -> Result<(), HistoryError> {
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

                Ok(())
            }
            Err(e) => Err(HistoryError::FileReadError { error: e }),
        }
    }

    pub fn save_history_file(&mut self, path: &str, mode: WriteMode) -> Result<(), HistoryError> {
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
                    self.append_index = self.entries.len();
                }
                Ok(())
            }
            Err(e) => Err(HistoryError::FileWriteError { error: e }),
        }
    }
}
