use std::fs;
use std::fs::{File, OpenOptions};
use std::path::Path;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum RedirectionMode {
    #[default]
    Nothing,
    Out,
    OutAppend,
    Error,
    ErrorAppend,
}

pub fn initialise_writer_file(mode: RedirectionMode, path: &str) -> File {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).ok();
    }

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(mode == RedirectionMode::Out || mode == RedirectionMode::Error)
        .append(mode == RedirectionMode::OutAppend || mode == RedirectionMode::ErrorAppend)
        .open(path)
        .unwrap()
}
