use crate::parser::Tokens;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub enum RedirectionMode {
    Output,
    Error,
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

pub fn write_output(
    output: &str,
    output_type: OutputType,
    tokens: &Tokens,
    writer: &mut impl Write,
) {
    if output.is_empty() {
        return;
    }

    match &tokens.redirection {
        Some(redirection) => match (output_type, &redirection.mode) {
            (OutputType::Stdout, RedirectionMode::Output) => write_to_file(output, redirection),
            (OutputType::Stderr, RedirectionMode::Error) => write_to_file(output, redirection),
            (OutputType::Stdout, _) => write_to_writer(output, writer),
            (OutputType::Stderr, _) => write_to_writer(output, writer),
        },
        None => match output_type {
            OutputType::Stdout => write_to_writer(output, writer),
            OutputType::Stderr => write_to_writer(output, writer),
        },
    }
}

pub fn write_to_writer(output: &str, writer: &mut impl Write) {
    writeln!(writer, "{}", output.trim()).unwrap();
}

pub fn write_to_file(output: &str, redirection: &Redirection) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&redirection.location)
        .unwrap();

    file.write_all(output.trim().as_bytes()).unwrap();
}
