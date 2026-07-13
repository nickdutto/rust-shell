use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use std::env;
use std::io::Write;

pub fn handle_pwd(mut io_streams: IoStreams) -> std::io::Result<ExitCode> {
    let mut final_exit_code = ExitCode::SUCCESS;

    match env::current_dir() {
        Ok(path) => {
            writeln!(io_streams.output, "{}", path.display().to_string().trim())?;
        }
        Err(e) => {
            writeln!(io_streams.error, "{}", e)?;
            final_exit_code = ExitCode::FAILURE;
        }
    }

    Ok(final_exit_code)
}
