use std::env;
use std::fs;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

fn search_path(command: &str) -> bool {
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
            if entry.file_name() == command.trim() {
                let metadata = entry.metadata().unwrap();
                if metadata.is_file() && (metadata.permissions().mode() & 0o111) != 0 {
                    println!(
                        "{} is {}",
                        command,
                        entry.path().as_os_str().to_str().unwrap()
                    );
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
                let command_in_path = search_path(&command);

                if !command_in_path {
                    println!("{}: not found", command);
                }
                continue;
            }
        } else {
            println!("{}: command not found", command);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_path() {
        search_path("ls");
        assert!(true);
    }
}
