use crate::io::stream::IoStreams;
use crate::parser::tokenize::Tokens;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub fn handle_complete(
    tokens: Tokens,
    shell_state: Arc<RwLock<ShellState>>,
    mut io_streams: IoStreams,
) {
    let mut args_iter = tokens.arguments.iter();

    while let Some(argument) = args_iter.next() {
        match argument.as_str() {
            "-C" => {
                if let Some(path_arg) = args_iter.next()
                    && let Some(name_arg) = args_iter.next()
                {
                    shell_state
                        .write()
                        .unwrap()
                        .completions
                        .insert(name_arg.clone(), path_arg.clone());
                }
            }
            "-r" => {
                if let Some(name_arg) = args_iter.next() {
                    shell_state.write().unwrap().completions.remove(name_arg);
                }
            }
            "-p" => {
                let Some(name_arg) = args_iter.next() else {
                    writeln!(
                        io_streams.error,
                        "complete: missing specification name for -p",
                    )
                    .unwrap();
                    return;
                };

                match shell_state
                    .read()
                    .unwrap()
                    .completions
                    .get_key_value(name_arg)
                {
                    Ok((name, path)) => {
                        writeln!(io_streams.output, "complete -C '{}' {}", path, name).unwrap()
                    }
                    Err(e) => writeln!(io_streams.error, "{}", e).unwrap(),
                }
            }
            _ => (),
        }
    }
}
