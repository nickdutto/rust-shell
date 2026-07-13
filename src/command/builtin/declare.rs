use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::shell::shell_state::ShellState;
use crate::shell::variables::VariableError;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub fn handle_declare(
    args: Vec<String>,
    shell_state: Arc<RwLock<ShellState>>,
    mut io_streams: IoStreams,
) -> std::io::Result<ExitCode> {
    let mut final_exit_code = ExitCode::SUCCESS;
    let mut args_iter = args.iter().peekable();

    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-p" => {
                if let Some(variable_key) = args_iter.next() {
                    if let Ok(Some((key, value))) = shell_state
                        .read()
                        .unwrap()
                        .variables
                        .get_key_value(variable_key)
                    {
                        writeln!(io_streams.output, "declare -- {}=\"{}\"", key, value)?;
                    } else {
                        writeln!(io_streams.error, "declare: {}: not found", variable_key)?;
                        final_exit_code = ExitCode::FAILURE;
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

                writeln!(io_streams.output, "{}", variables.trim_end())?;
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
                    )?;
                    final_exit_code = ExitCode::FAILURE;
                }
            }
        }
    }

    Ok(final_exit_code)
}
