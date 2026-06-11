use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::shell::ShellState;

pub fn handle_complete(tokens: Tokens, shell_state: &mut ShellState) {
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
                    write_output(
                        format!("{}: missing specification name for -p", tokens.command).trim(),
                        OutputType::Stderr,
                        &tokens,
                    );
                    return;
                };

                if let Some((spec_name, spec_path)) = shell_state
                    .completion_specifications
                    .get_key_value(spec_name_arg)
                {
                    write_output(
                        format!("{} -C '{}' {}", tokens.command, spec_path, spec_name).trim(),
                        OutputType::Stdout,
                        &tokens,
                    );
                } else {
                    write_output(
                        format!(
                            "{}: {}: no completion specification",
                            tokens.command, spec_name_arg
                        )
                            .trim(),
                        OutputType::Stderr,
                        &tokens,
                    );
                    return;
                };
            }
            _ => (),
        }
    }
}
