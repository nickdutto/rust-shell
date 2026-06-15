use crate::io::stream::IoStreams;
use crate::io::tokenize::Tokens;
use crate::shell::shell_state::ShellState;
use std::io::Write;

pub fn handle_complete(tokens: Tokens, shell_state: &mut ShellState, mut io_streams: IoStreams) {
    let mut arguments_iter = tokens.arguments.iter();

    while let Some(argument) = arguments_iter.next() {
        match argument.as_str() {
            "-C" => {
                if let Some(spec_path_arg) = arguments_iter.next()
                    && let Some(spec_name_arg) = arguments_iter.next()
                {
                    shell_state
                        .completion_specifications
                        .insert(spec_name_arg.clone(), spec_path_arg.clone());
                }
            }
            "-r" => {
                if let Some(spec_name_arg) = arguments_iter.next() {
                    shell_state.completion_specifications.remove(spec_name_arg);
                }
            }
            "-p" => {
                let Some(spec_name_arg) = arguments_iter.next() else {
                    writeln!(
                        io_streams.error,
                        "complete: missing specification name for -p",
                    )
                        .unwrap();
                    return;
                };

                if let Some((spec_name, spec_path)) = shell_state
                    .completion_specifications
                    .get_key_value(spec_name_arg)
                {
                    writeln!(
                        io_streams.output,
                        "complete -C '{}' {}",
                        spec_path, spec_name
                    )
                        .unwrap();
                } else {
                    writeln!(
                        io_streams.error,
                        "complete: {}: no completion specification",
                        spec_name_arg,
                    )
                        .unwrap();
                    return;
                };
            }
            _ => (),
        }
    }
}
