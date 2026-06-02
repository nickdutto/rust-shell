use crate::env::get_env_paths;
use crate::parser::{Tokens, tokenize_arguments};
use crate::writer::{OutputType, write_output};
use std::io::{BufReader, ErrorKind, Read, Write, stdout};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use std::{env, process};

pub enum Command {
    Cd(Tokens),
    Echo(Tokens),
    Executable(Tokens),
    Exit,
    Pwd(Tokens),
    Type(Tokens),
}

impl Command {
    pub fn parse_command(input: &str) -> Command {
        let tokens = tokenize_arguments(input.trim());

        match tokens.command.as_str() {
            "cd" => Command::Cd(tokens),
            "echo" => Command::Echo(tokens),
            "exit" => Command::Exit,
            "pwd" => Command::Pwd(tokens),
            "type" => Command::Type(tokens),
            _ => Command::Executable(tokens),
        }
    }

    pub fn run_command(command: Command, out_writer: &mut impl Write, err_writer: &mut impl Write) {
        match command {
            Command::Cd(tokens) => handle_cd(tokens, err_writer),
            Command::Echo(tokens) => handle_echo(tokens, out_writer),
            Command::Executable(tokens) => handle_executable(tokens, out_writer, err_writer),
            Command::Exit => handle_exit(),
            Command::Pwd(tokens) => handle_pwd(tokens, out_writer),
            Command::Type(tokens) => handle_type(tokens, out_writer, err_writer),
        }
    }
}

fn handle_cd(tokens: Tokens, err_writer: &mut impl Write) {
    let target = tokens
        .arguments
        .first()
        .map(|s| s.as_str().trim())
        .unwrap_or("~");

    let result = match target {
        "~" => {
            if let Some(home) = env::var_os("HOME") {
                cd_set_dir(Path::new(&home))
            } else {
                Ok(())
            }
        }
        _ => cd_set_dir(Path::new(&target)),
    };

    if let Err(err_message) = result {
        write_output(&err_message, OutputType::Stderr, &tokens, err_writer);
    }
}

fn handle_echo(tokens: Tokens, out_writer: &mut impl Write) {
    let output = tokens.arguments.join(" ");
    write_output(output.trim(), OutputType::Stdout, &tokens, out_writer);
}

fn handle_executable(tokens: Tokens, out_writer: &mut impl Write, err_writer: &mut impl Write) {
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

fn handle_exit() {
    stdout().flush().unwrap();
    process::exit(0);
}

fn handle_pwd(tokens: Tokens, out_writer: &mut impl Write) {
    let path = env::current_dir().unwrap();
    write_output(
        &format!("{}", path.display()),
        OutputType::Stdout,
        &tokens,
        out_writer,
    );
}

fn handle_type(tokens: Tokens, out_writer: &mut impl Write, err_writer: &mut impl Write) {
    let builtin_commands = ["cd", "echo", "exit", "pwd", "type"];
    let command_name = match tokens.arguments.first() {
        Some(command) => command.as_str().trim(),
        None => {
            write_output(
                "type: missing operand",
                OutputType::Stderr,
                &tokens,
                err_writer,
            );
            return;
        }
    };

    let mut output = String::new();

    if builtin_commands.contains(&command_name) {
        output = format!("{} is a shell builtin", command_name);
    } else {
        let paths = match get_env_paths("PATH") {
            Ok(paths) => paths,
            Err(err_message) => {
                write_output(&err_message, OutputType::Stderr, &tokens, err_writer);
                vec![]
            }
        };

        let entries = paths
            .into_iter()
            .filter_map(|dir_path| dir_path.read_dir().ok())
            .flatten()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter_map(|entry| {
                if let Ok(metadata) = entry.metadata()
                    && metadata.is_file()
                    // TODO: permissions check is not cross platform
                    && (metadata.permissions().mode() & 0o111) != 0
                {
                    return Some(entry);
                }

                None
            });

        let mut found = false;
        for entry in entries {
            if entry.file_name().unwrap().to_str().unwrap() != command_name {
                continue;
            }

            output = format!("{} is {}", command_name, entry.to_str().unwrap());
            found = true;
            break;
        }

        if !found {
            output = format!("{}: not found", command_name);
        }
    }

    write_output(&output, OutputType::Stdout, &tokens, out_writer);
}

fn cd_set_dir(path: &Path) -> Result<(), String> {
    match env::set_current_dir(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Err(format!(
            "cd: {}: No such file or directory",
            path.to_str().unwrap()
        )),
        Err(e) => Err(format!("cd: {}: {}", path.display(), e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_echo_output() {
        let mut stdout_buffer = Vec::new();
        let tokens = Tokens {
            command: "echo".to_string(),
            arguments: vec!["hello".to_string(), "world".to_string()],
            redirection: None,
        };

        handle_echo(tokens, &mut stdout_buffer);
        let result = String::from_utf8(stdout_buffer).unwrap();

        assert_eq!(result, "hello world\n");
    }

    #[test]
    fn test_handle_type_builtin() {
        let mut stdout_buffer = Vec::new();
        let mut stderr_buffer = Vec::new();
        let tokens = Tokens {
            command: "type".to_string(),
            arguments: vec!["exit".to_string()],
            redirection: None,
        };

        handle_type(tokens, &mut stdout_buffer, &mut stderr_buffer);
        let result = String::from_utf8(stdout_buffer).unwrap();

        assert_eq!(result, "exit is a shell builtin\n");
        assert!(stderr_buffer.is_empty());
    }

    #[test]
    fn test_handle_type_via_path() {
        let mut stdout_buffer = Vec::new();
        let mut stderr_buffer = Vec::new();
        let tokens = Tokens {
            command: "type".to_string(),
            arguments: vec!["ls".to_string()],
            redirection: None,
        };

        handle_type(tokens, &mut stdout_buffer, &mut stderr_buffer);
        let result = String::from_utf8(stdout_buffer).unwrap();

        // TODO: this relies on the host machine having ls available so its not really a clean test
        assert!(result.contains("ls is /"));
        assert!(stderr_buffer.is_empty());
    }

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
