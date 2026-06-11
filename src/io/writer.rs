use crate::io::tokenize::Tokens;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
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

pub enum OutputType {
    Stdout,
    Stderr,
}

pub fn write_output(output: &str, output_type: OutputType, tokens: &Tokens) {
    if output.is_empty() {
        return;
    }

    match &tokens.redirection {
        Some(redirection) => match (&output_type, &redirection.mode) {
            (OutputType::Stdout, RedirectionMode::Output)
            | (OutputType::Stdout, RedirectionMode::OutputAppend) => {
                write_to_file(output, redirection)
            }
            (OutputType::Stderr, RedirectionMode::Error)
            | (OutputType::Stderr, RedirectionMode::ErrorAppend) => {
                write_to_file(output, redirection)
            }
            (OutputType::Stdout, _) => write_to_writer(output, &output_type),
            (OutputType::Stderr, _) => write_to_writer(output, &output_type),
        },
        None => match output_type {
            OutputType::Stdout => write_to_writer(output, &output_type),
            OutputType::Stderr => write_to_writer(output, &output_type),
        },
    }
}

pub fn write_to_writer(output: &str, output_type: &OutputType) {
    match output_type {
        OutputType::Stdout => println!("{}", output),
        OutputType::Stderr => eprintln!("{}", output),
    }
}

pub fn write_to_file(output: &str, redirection: &Redirection) {
    let mut file = initialise_writer_file(redirection);

    if output.ends_with('\n') {
        file.write_all(output.as_bytes()).unwrap();
    } else {
        file.write_all(format!("{}\n", output).as_bytes()).unwrap();
    }
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
