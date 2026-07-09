use crate::io::stream::{IoStreams, OutputStream};
use std::io::ErrorKind;
use std::io::Write;
use std::process::{Child, Command};

pub fn handle_executable(
    cmd: &str,
    args: Vec<String>,
    io_streams: IoStreams,
) -> Result<Child, i32> {
    if cmd.is_empty() {
        return Err(0);
    }

    let mut fallback_error = OutputStream::fallback_output_stream(&io_streams.error);

    let mut command_binding = Command::new(cmd);
    let command = command_binding
        .stdin(io_streams.input.into_stdio())
        .stdout(io_streams.output.into_stdio())
        .stderr(io_streams.error.into_stdio());

    match command.args(&args).spawn() {
        Ok(child) => Ok(child),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            writeln!(fallback_error, "{}: command not found", cmd).ok();
            Err(127)
        }
        Err(e) => {
            writeln!(fallback_error, "{}: error executing command: {}", cmd, e).ok();
            Err(1)
        }
    }
}
