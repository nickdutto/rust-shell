use crate::command::command::BUILTIN_COMMANDS;
use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::system::env::get_env_paths;
use is_executable::is_executable;

pub fn handle_type(tokens: Tokens) {
    let command_name = match tokens.arguments.first() {
        Some(command) => command.as_str().trim(),
        None => {
            write_output("type: missing operand", OutputType::Stderr, &tokens);
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
                write_output(&err_message, OutputType::Stderr, &tokens);
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

    write_output(&output, OutputType::Stdout, &tokens);
}
