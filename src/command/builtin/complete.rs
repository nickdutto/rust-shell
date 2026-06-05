use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::shell::ShellState;
use std::io::Write;

pub fn handle_complete(
    tokens: Tokens,
    shell_state: &mut ShellState,
    out_writer: &mut impl Write,
    err_writer: &mut impl Write,
) {
    let mut arguments_iter = tokens.arguments.iter();

    while let Some(argument) = arguments_iter.next() {
        match argument.as_str() {
            "-p" => {
                let Some(print) = arguments_iter.next() else {
                    write_output(
                        &format!(
                            "{}: no completion specification name provided for -p",
                            tokens.command
                        ),
                        OutputType::Stderr,
                        &tokens,
                        err_writer,
                    );
                    return;
                };

                let Some(specification) = shell_state.completion_specifications.get(print) else {
                    write_output(
                        &format!("{}: {}: no completion specification", tokens.command, print),
                        OutputType::Stderr,
                        &tokens,
                        err_writer,
                    );
                    return;
                };

                write_output(
                    &format!("{} -C '{}' {}", tokens.command, specification, print),
                    OutputType::Stdout,
                    &tokens,
                    out_writer,
                );
            }
            "-C" => {
                let register_path = arguments_iter.next();
                let register_name = arguments_iter.next();

                if let Some(path) = register_path
                    && let Some(name) = register_name
                {
                    shell_state
                        .completion_specifications
                        .insert(name.clone(), path.clone());
                }
            }
            _ => (),
        }
    }
}
