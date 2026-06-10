use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use std::io::{BufReader, ErrorKind, Read};
use std::process;
use std::process::{Child, Stdio};

pub fn handle_executable(tokens: Tokens) {
    if tokens.command.is_empty() {
        return;
    }

    let mut command_binding = process::Command::new(&tokens.command);
    let command = command_binding
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if tokens
        .arguments
        .iter()
        .next_back()
        .unwrap_or(&"".to_string())
        == "&"
    {
        match_spawn_process(
            command
                .args(&tokens.arguments[..tokens.arguments.len() - 1])
                .spawn(),
            true,
            tokens,
        );
    } else {
        match_spawn_process(command.args(&tokens.arguments).spawn(), false, tokens)
    }

    fn match_spawn_process(
        result: std::io::Result<Child>,
        run_background_job: bool,
        tokens: Tokens,
    ) {
        let mut stdout_output = String::new();
        let mut stderr_output = String::new();

        match result {
            Ok(mut child) => {
                if run_background_job {
                    write_output(&format!("[1] {}", child.id()), OutputType::Stdout, &tokens);

                    std::thread::spawn(move || {
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

                        println!("hello world");
                    });
                } else {
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
}
