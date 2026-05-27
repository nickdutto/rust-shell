use std::env;
use std::fmt::format;
use std::fs;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

enum PathCommandMode {
    Type,
    Execute,
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
                            Command::new(entry.path().as_os_str())
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

fn main() {
    let builtin_commands = ["echo", "exit", "type"];

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        command = command.trim().to_string();
        if command == "exit" {
            break;
        } else if command.starts_with("echo") {
            command = command[5..].trim().to_string();
            println!("{}", command);
        } else if command.starts_with("type") {
            command = command[5..].trim().to_string();
            if builtin_commands.contains(&command.as_str()) {
                println!("{} is a shell builtin", command);
            } else {
                let command_in_path = run_command_by_path(&command, PathCommandMode::Type);

                if !command_in_path {
                    println!("{}: not found", command);
                }
                continue;
            }
        } else {
            let command_in_path = run_command_by_path(&command, PathCommandMode::Execute);

            if !command_in_path {
                println!("{}: command not found", command);
            }
        }
    }
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
