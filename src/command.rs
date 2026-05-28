use crate::env::get_env_paths;
use std::io::{ErrorKind, Write, stdout};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{env, process};

pub enum Command {
    Cd(String),
    Exit,
    Echo(String),
    Executable(String),
    Type(String),
    Pwd,
}

impl Command {
    pub fn parse_command(input: &str) -> Command {
        let input = input.trim();
        match input {
            s if s[..2].to_string() == "cd" => Command::Cd(input.into()),
            "exit" => Command::Exit,
            "pwd" => Command::Pwd,
            s if s.starts_with("echo") => Command::Echo(input.into()),
            s if s.starts_with("type") => Command::Type(input.into()),
            _ => Command::Executable(input.into()),
        }
    }

    pub fn run_command(command: Command, writer: &mut impl Write) {
        match command {
            Command::Cd(cmd) => handle_cd(&cmd, writer),
            Command::Exit => handle_exit(),
            Command::Echo(cmd) => handle_echo(&cmd, writer),
            Command::Executable(cmd) => handle_executable(&cmd, writer),
            Command::Type(cmd) => handle_type(&cmd, writer),
            Command::Pwd => handle_pwd(writer),
        }
    }
}

fn handle_cd(input: &str, writer: &mut impl Write) {
    let path = Path::new(&input[3..]);
    match env::set_current_dir(path) {
        Ok(_) => (),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            writeln!(
                writer,
                "cd: {}: No such file or directory",
                path.to_str().unwrap().trim(),
            )
            .unwrap();
        }
        Err(e) => {
            writeln!(writer, "cd: {}: {}", path.to_str().unwrap().trim(), e).unwrap();
        }
    }
}

fn handle_exit() {
    stdout().flush().unwrap();
    process::exit(0);
}

fn handle_echo(input: &str, writer: &mut impl Write) {
    writeln!(writer, "{}", input[5..].trim()).unwrap();
}

fn handle_pwd(writer: &mut impl Write) {
    let path = env::current_dir().unwrap();
    writeln!(writer, "{}", path.to_str().unwrap()).unwrap();
}

fn handle_executable(input: &str, writer: &mut impl Write) {
    let command_split: Vec<&str> = input.trim().split(" ").collect();
    let command_name = command_split.first().unwrap();
    let command_args = command_split.get(1..).unwrap();

    match process::Command::new(command_name)
        .args(command_args)
        .spawn()
    {
        Ok(mut child) => {
            child.wait().unwrap();
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            writeln!(writer, "{}: command not found", command_name.trim()).unwrap();
        }
        Err(e) => {
            writeln!(
                writer,
                "{}: error executing command: {}",
                command_name.trim(),
                e
            )
            .unwrap();
        }
    }
}

fn handle_type(input: &str, writer: &mut impl Write) {
    let builtin_commands = ["cd", "echo", "exit", "type", "pwd"];
    let command = input[5..].trim().to_string();

    if builtin_commands.contains(&command.as_str()) {
        writeln!(writer, "{} is a shell builtin", command).unwrap();
    } else {
        let command_split: Vec<&str> = command.trim().split(" ").collect();
        let command_name = command_split.first().unwrap();

        let entries = get_env_paths("PATH", writer)
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

        for entry in entries {
            if entry.file_name().unwrap().to_str().unwrap() != *command_name {
                continue;
            }

            writeln!(writer, "{} is {}", command, entry.to_str().unwrap()).unwrap();
            return;
        }

        writeln!(writer, "{}: not found", command).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_echo_output() {
        let mut buffer = Vec::new();
        handle_echo("echo hello world", &mut buffer);
        let result = String::from_utf8(buffer).unwrap();

        assert_eq!(result, "hello world\n");
    }

    #[test]
    fn test_handle_type_builtin() {
        let mut buffer = Vec::new();
        handle_type("type exit", &mut buffer);
        let result = String::from_utf8(buffer).unwrap();

        assert_eq!(result, "exit is a shell builtin\n");
    }

    #[test]
    fn test_handle_type_via_path() {
        let mut buffer = Vec::new();
        handle_type("type ls", &mut buffer);
        let result = String::from_utf8(buffer).unwrap();

        // TODO: this relies on the host machine having ls available so its not really a clean test
        assert!(result.contains("ls is /"));
    }

    #[test]
    fn test_handle_executable_missing() {
        let mut buffer = Vec::new();
        handle_executable("not_a_real_command_1234", &mut buffer);
        let result = String::from_utf8(buffer).unwrap();

        assert_eq!(result, "not_a_real_command_1234: command not found\n");
    }
}
