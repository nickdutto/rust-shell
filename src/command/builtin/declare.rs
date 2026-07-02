use crate::io::stream::IoStreams;
use crate::parser::tokenize::Tokens;
use crate::shell::shell_state::ShellState;
use crate::shell::variables::VariableError;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub fn handle_declare(
    tokens: Tokens,
    shell_state: Arc<RwLock<ShellState>>,
    mut io_streams: IoStreams,
) {
    let mut arguments_iter = tokens.arguments.iter().peekable();
    while let Some(argument) = arguments_iter.next() {
        match argument.as_str() {
            "-p" => {
                if let Some(variable_key) = arguments_iter.peek() {
                    if let Ok(Some((key, value))) = shell_state
                        .read()
                        .unwrap()
                        .variables
                        .get_key_value(variable_key)
                    {
                        writeln!(io_streams.output, "declare -- {}=\"{}\"", key, value).unwrap();
                    } else {
                        writeln!(io_streams.error, "declare: {}: not found", variable_key).unwrap();
                    }
                }
            }
            "-l" => {
                let variables: String = shell_state
                    .read()
                    .unwrap()
                    .variables
                    .iter()
                    .map(|(key, value)| format!("{}=\"{}\"\n", key, value))
                    .collect();

                writeln!(io_streams.output, "{}", variables.trim_end()).unwrap();
            }
            variable_arg => {
                if let Some((key, value)) = variable_arg.split_once('=')
                    && let Err(VariableError::InvalidIdentifier { key, value }) = shell_state
                        .write()
                        .unwrap()
                        .variables
                        .insert(key.to_string(), value.to_string())
                {
                    writeln!(
                        io_streams.error,
                        "declare: `{}={}': not a valid identifier",
                        key, value,
                    )
                    .unwrap();
                }
            }
        }
    }
}
