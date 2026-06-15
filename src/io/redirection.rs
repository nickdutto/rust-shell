use std::fs;
use std::fs::{File, OpenOptions};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum RedirectionMode {
    Output,
    OutputAppend,
    Error,
    ErrorAppend,
}

#[derive(Debug)]
pub struct Redirection {
    pub mode: RedirectionMode,
    pub location: String,
}

pub fn initialise_writer_file(redirection: &Redirection) -> File {
    if let Some(parent) = Path::new(&redirection.location).parent() {
        fs::create_dir_all(parent).ok();
    }

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(
            redirection.mode == RedirectionMode::Output
                || redirection.mode == RedirectionMode::Error,
        )
        .append(
            redirection.mode == RedirectionMode::OutputAppend
                || redirection.mode == RedirectionMode::ErrorAppend,
        )
        .open(&redirection.location)
        .unwrap()
}
