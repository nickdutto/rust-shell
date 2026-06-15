use crate::io::stream::{IoStreams, OutputStream};
use crate::io::tokenize::Tokens;
use crate::shell::jobs::{BackgroundJob, BackgroundJobStatus};
use crate::shell::shell_state::ShellState;
use std::io::ErrorKind;
use std::io::Write;
use std::process::{Child, Command};
use std::sync::{Arc, RwLock};

pub fn handle_executable(
    tokens: Tokens,
    shell_state: Arc<RwLock<ShellState>>,
    io_streams: IoStreams,
) -> Option<Child> {
    if tokens.command.is_empty() {
        return None;
    }

    let mut fallback_output = OutputStream::fallback_output_stream(&io_streams.output);
    let mut fallback_error = OutputStream::fallback_output_stream(&io_streams.error);

    let mut command_binding = Command::new(&tokens.command);
    let command = command_binding
        .stdin(io_streams.input.into_stdio())
        .stdout(io_streams.output.into_stdio())
        .stderr(io_streams.error.into_stdio());

    let is_background = tokens.arguments.last().map(|s| s.as_str()) == Some("&");
    let args = if is_background {
        &tokens.arguments[..tokens.arguments.len() - 1]
    } else {
        &tokens.arguments
    };

    match command.args(args).spawn() {
        Ok(mut child) => {
            if is_background {
                let shell_state_clone = Arc::clone(&shell_state);
                let pid = child.id();

                let job_id = {
                    let mut guard = shell_state_clone.write().unwrap();
                    let id = (1..)
                        .find(|&smallest| {
                            !guard.background_jobs.iter().any(|job| job.id == smallest)
                        })
                        .unwrap();

                    guard.background_jobs.push(BackgroundJob {
                        id,
                        pid,
                        command: format!("{} {}", tokens.command, tokens.arguments.join(" ")),
                        status: BackgroundJobStatus::Running,
                    });
                    id
                };

                writeln!(fallback_output, "[{}] {}", job_id, pid).unwrap();

                std::thread::spawn(move || {
                    child.wait().unwrap();
                    let mut guard = shell_state_clone.write().unwrap();
                    if let Some(job) = guard
                        .background_jobs
                        .iter_mut()
                        .find(|job| job.id == job_id)
                    {
                        job.status = BackgroundJobStatus::Done;
                        if let Some(stripped) = job.command.strip_suffix(" &") {
                            job.command = stripped.to_string();
                        }
                    }
                });

                None
            } else {
                Some(child)
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            writeln!(fallback_error, "{}: command not found", tokens.command).unwrap();
            None
        }
        Err(e) => {
            writeln!(
                fallback_error,
                "{}: error executing command: {}",
                tokens.command, e
            )
                .unwrap();
            None
        }
    }
}
