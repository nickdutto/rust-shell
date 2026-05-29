use crate::env::get_env_paths;
use crate::parser::tokenize_arguments;
use std::io::{ErrorKind, Write, stdout};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{env, process};

pub enum Command {
    Cd(Vec<String>),
    Echo(Vec<String>),
    Executable(Vec<String>),
    Exit,
    Pwd,
    Type(Vec<String>),
}

impl Command {
    pub fn parse_command(input: &str) -> Command {
        let input = tokenize_arguments(input.trim());

        if input.is_empty() {
            return Command::Executable(vec![]);
        }

        let command = input.first().unwrap().trim();
        let args = input.get(1..).unwrap_or(&[]).to_vec();

        match command {
            "cd" => Command::Cd(args),
            "echo" => Command::Echo(args),
            "exit" => Command::Exit,
            "pwd" => Command::Pwd,
            "type" => Command::Type(args),
            _ => Command::Executable(input),
        }
    }

    pub fn run_command(command: Command, writer: &mut impl Write) {
        match command {
            Command::Cd(args) => handle_cd(&args, writer),
            Command::Echo(args) => handle_echo(&args, writer),
            Command::Executable(args) => handle_executable(&args, writer),
            Command::Exit => handle_exit(),
            Command::Pwd => handle_pwd(writer),
            Command::Type(args) => handle_type(&args, writer),
        }
    }
}

fn handle_cd(args: &[String], writer: &mut impl Write) {
    let target = args.first().map(|s| s.as_str().trim()).unwrap_or("~");
    match target {
        "~" => {
            if let Some(home) = env::var_os("HOME") {
                cd_set_dir(Path::new(&home), writer);
            }
        }
        _ => cd_set_dir(Path::new(&target), writer),
    }
}

fn handle_echo(args: &[String], writer: &mut impl Write) {
    let output = args.join(" ");
    writeln!(writer, "{}", output.trim()).unwrap();
}

fn handle_executable(tokens: &[String], writer: &mut impl Write) {
    if tokens.is_empty() {
        return;
    }

    let command_name = tokens.first().unwrap().trim();
    let command_args = tokens.get(1..).unwrap();

    match process::Command::new(command_name)
        .args(command_args)
        .spawn()
    {
        Ok(mut child) => {
            child.wait().unwrap();
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            writeln!(writer, "{}: command not found", command_name).unwrap();
        }
        Err(e) => {
            writeln!(writer, "{}: error executing command: {}", command_name, e).unwrap();
        }
    }
}

fn handle_exit() {
    stdout().flush().unwrap();
    process::exit(0);
}

fn handle_pwd(writer: &mut impl Write) {
    let path = env::current_dir().unwrap();
    writeln!(writer, "{}", path.to_str().unwrap()).unwrap();
}

fn handle_type(args: &[String], writer: &mut impl Write) {
    let builtin_commands = ["cd", "echo", "exit", "pwd", "type"];
    let command_name = match args.first() {
        Some(command) => command.as_str().trim(),
        None => {
            writeln!(writer, "type: missing operand").unwrap();
            return;
        }
    };

    if builtin_commands.contains(&command_name) {
        writeln!(writer, "{} is a shell builtin", command_name).unwrap();
    } else {
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
            if entry.file_name().unwrap().to_str().unwrap() != command_name {
                continue;
            }

            writeln!(writer, "{} is {}", command_name, entry.to_str().unwrap()).unwrap();
            return;
        }

        writeln!(writer, "{}: not found", command_name).unwrap();
    }
}

fn cd_set_dir(path: &Path, writer: &mut impl Write) {
    match env::set_current_dir(path) {
        Ok(_) => (),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            writeln!(
                writer,
                "cd: {}: No such file or directory",
                path.to_str().unwrap(),
            )
            .unwrap();
        }
        Err(e) => {
            writeln!(writer, "cd: {}: {}", path.to_str().unwrap(), e).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_echo_output() {
        let mut buffer = Vec::new();
        let args = vec!["hello".to_string(), "world".to_string()];
        handle_echo(&args, &mut buffer);
        let result = String::from_utf8(buffer).unwrap();

        assert_eq!(result, "hello world\n");
    }

    #[test]
    fn test_handle_type_builtin() {
        let mut buffer = Vec::new();
        let args = vec!["exit".to_string()];
        handle_type(&args, &mut buffer);
        let result = String::from_utf8(buffer).unwrap();

        assert_eq!(result, "exit is a shell builtin\n");
    }

    #[test]
    fn test_handle_type_via_path() {
        let mut buffer = Vec::new();
        let args = vec!["ls".to_string()];
        handle_type(&args, &mut buffer);
        let result = String::from_utf8(buffer).unwrap();

        // TODO: this relies on the host machine having ls available so its not really a clean test
        assert!(result.contains("ls is /"));
    }

    #[test]
    fn test_handle_executable_missing() {
        let mut buffer = Vec::new();
        let tokens = vec!["not_a_real_command_1234".to_string()];
        handle_executable(&tokens, &mut buffer);
        let result = String::from_utf8(buffer).unwrap();

        assert_eq!(result, "not_a_real_command_1234: command not found\n");
    }
}
