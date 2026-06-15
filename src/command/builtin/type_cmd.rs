use crate::command::command::BUILTIN_COMMANDS;
use crate::io::stream::IoStreams;
use crate::io::tokenize::Tokens;
use crate::system::env::get_env_paths;
use is_executable::is_executable;
use std::io::Write;

pub fn handle_type(tokens: Tokens, mut io_streams: IoStreams) {
    let command_name = match tokens.arguments.first() {
        Some(command) => command.as_str().trim(),
        None => {
            writeln!(io_streams.error, "type: missing operand").unwrap();
            return;
        }
    };

    let mut output = String::new();

    if BUILTIN_COMMANDS.contains(&command_name) {
        output = format!("{} is a shell builtin", command_name);
    } else {
        let paths = match get_env_paths("PATH") {
            Ok(paths) => paths,
            Err(e) => {
                writeln!(io_streams.error, "{}", e.trim()).unwrap();
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

    writeln!(io_streams.output, "{}", output.trim()).unwrap();
}
