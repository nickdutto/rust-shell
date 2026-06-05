use crate::command::command::BUILTIN_COMMANDS;
use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::system::env::get_env_paths;
use is_executable::is_executable;
use std::io::Write;

pub fn handle_type(tokens: Tokens, out_writer: &mut impl Write, err_writer: &mut impl Write) {
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

    if BUILTIN_COMMANDS.contains(&command_name) {
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
            .filter_map(|path| {
                if is_executable(&path) {
                    return Some(path);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::tokenize::Tokens;

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
}
