use crate::engine::command::BUILTIN_COMMANDS;
use crate::io::stream::IoStreams;
use crate::system::env::get_env_paths;
use is_executable::is_executable;
use std::io::Write;

pub fn handle_type(cmd: &str, mut io_streams: IoStreams) {
    if cmd.is_empty() {
        writeln!(io_streams.error, "type: missing operand").unwrap();
        return;
    }

    let mut output = String::new();

    if BUILTIN_COMMANDS.contains(&cmd) {
        output = format!("{} is a shell builtin", cmd);
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
            if entry.file_name().unwrap().to_str().unwrap() != cmd {
                continue;
            }

            output = format!("{} is {}", cmd, entry.to_str().unwrap());
            found = true;
            break;
        }

        if !found {
            output = format!("{}: not found", cmd);
        }
    }

    writeln!(io_streams.output, "{}", output.trim()).unwrap();
}
