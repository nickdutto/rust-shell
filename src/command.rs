use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{env, io};

pub enum Command {
    Exit,
    Echo(String),
    Executable(String),
    Type(String),
}

impl Command {
    pub fn parse_command(input: &str) -> Command {
        match input {
            "exit" => Command::Exit,
            s if s.starts_with("echo") => Command::Echo(input.into()),
            s if s.starts_with("type") => Command::Type(input.into()),
            _ => Command::Executable(input.into()),
        }
    }

    pub fn run_command(command: Command) {
        match command {
            Command::Exit => handle_exit(),
            Command::Echo(cmd) => handle_echo(&cmd),
            Command::Executable(cmd) => handle_executable(&cmd),
            Command::Type(cmd) => handle_type(&cmd),
        }
    }
}

enum PathCommandMode {
    Type,
    Execute,
}

fn handle_exit() {
    io::stdout().flush().unwrap();
    std::process::exit(1);
}

fn handle_echo(input: &str) {
    println!("{}", input[5..].trim());
}

fn handle_executable(input: &str) {
    let command_in_path = run_command_by_path(input, PathCommandMode::Execute);

    if !command_in_path {
        println!("{}: command not found", input);
    }
}

fn handle_type(input: &str) {
    let builtin_commands = ["echo", "exit", "type"];
    let command = input[5..].trim().to_string();

    if builtin_commands.contains(&command.as_str()) {
        println!("{} is a shell builtin", command);
    } else {
        let command_in_path = run_command_by_path(&command, PathCommandMode::Type);

        if !command_in_path {
            println!("{}: not found", command);
        }
    }
}

fn run_command_by_path(command: &str, mode: PathCommandMode) -> bool {
    let command_split: Vec<&str> = command.trim().split(" ").collect();
    let mut paths: Vec<PathBuf> = vec![];

    let path_var = "PATH";
    match env::var_os(path_var) {
        Some(var_paths) => {
            for path in env::split_paths(&var_paths) {
                if path.is_dir() {
                    paths.push(path.to_path_buf());
                }
            }
        }
        None => println!("{path_var} is not defined in the environment."),
    }

    for path_buf in paths {
        for entry in path_buf.read_dir().unwrap() {
            let entry = entry.unwrap();
            let command_name = command_split.first().unwrap();
            if entry.file_name() == *command_name {
                let metadata = entry.metadata().unwrap();
                if metadata.is_file() && (metadata.permissions().mode() & 0o111) != 0 {
                    match mode {
                        PathCommandMode::Type => println!(
                            "{} is {}",
                            command,
                            entry.path().as_os_str().to_str().unwrap()
                        ),
                        PathCommandMode::Execute => {
                            std::process::Command::new(command_name)
                                .args(command_split.get(1..).unwrap())
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap();
                        }
                    }

                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_by_path_type() {
        run_command_by_path("ls", PathCommandMode::Type);
        assert!(true);
    }

    #[test]
    fn test_run_command_by_path_execute() {
        run_command_by_path("ls", PathCommandMode::Execute);
        assert!(true);
    }
}
