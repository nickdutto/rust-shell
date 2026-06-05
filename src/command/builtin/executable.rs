use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use std::io::{BufReader, ErrorKind, Read, Write};
use std::process;
use std::process::Stdio;

pub fn handle_executable(tokens: Tokens, out_writer: &mut impl Write, err_writer: &mut impl Write) {
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

            write_output(&stdout_output, OutputType::Stdout, &tokens, out_writer);
            write_output(&stderr_output, OutputType::Stderr, &tokens, err_writer);
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            write_output(
                &format!("{}: command not found", tokens.command),
                OutputType::Stderr,
                &tokens,
                err_writer,
            );
        }
        Err(e) => {
            write_output(
                &format!("{}: error executing command: {}", tokens.command, e),
                OutputType::Stderr,
                &tokens,
                err_writer,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::tokenize::Tokens;

    #[test]
    fn test_handle_executable_missing() {
        let mut stdout_buffer = Vec::new();
        let mut stderr_buffer = Vec::new();
        let tokens = Tokens {
            command: "not_a_real_command_1234".to_string(),
            arguments: vec![],
            redirection: None,
        };

        handle_executable(tokens, &mut stdout_buffer, &mut stderr_buffer);
        let result = String::from_utf8(stderr_buffer).unwrap();

        assert_eq!(result, "not_a_real_command_1234: command not found\n");
        assert!(stdout_buffer.is_empty());
    }
}
