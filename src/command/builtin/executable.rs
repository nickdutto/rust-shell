use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use std::io::{BufReader, ErrorKind, Read};
use std::process;
use std::process::Stdio;

pub fn handle_executable(tokens: Tokens) {
    if tokens.command.is_empty() {
        return;
    }

    let mut stdout_output = String::new();
    let mut stderr_output = String::new();

    match process::Command::new(&tokens.command)
        .args(&tokens.arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout);
                reader.read_to_string(&mut stdout_output).unwrap();
            }

            if let Some(stderr) = child.stderr.take() {
                let mut reader = BufReader::new(stderr);
                reader.read_to_string(&mut stderr_output).unwrap();
            }

            child.wait().unwrap();

            write_output(&stdout_output, OutputType::Stdout, &tokens);
            write_output(&stderr_output, OutputType::Stderr, &tokens);
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            write_output(
                &format!("{}: command not found", tokens.command),
                OutputType::Stderr,
                &tokens,
            );
        }
        Err(e) => {
            write_output(
                &format!("{}: error executing command: {}", tokens.command, e),
                OutputType::Stderr,
                &tokens,
            );
        }
    }
}
