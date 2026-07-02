use crate::io::stream::{IoStreams, OutputStream};
use crate::parser::tokenize::Tokens;
use crate::shell::background_jobs::{BackgroundJob, BackgroundJobStatus};
use crate::shell::shell_state::ShellState;
use reedline::ExternalPrinter;
use std::io::{BufRead, BufReader, ErrorKind};
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, RwLock};

pub fn handle_executable(
    tokens: Tokens,
    shell_state: Arc<RwLock<ShellState>>,
    io_streams: IoStreams,
    printer: ExternalPrinter<String>,
) -> Option<Child> {
    if tokens.command.is_empty() {
        return None;
    }

    let mut fallback_error = OutputStream::fallback_output_stream(&io_streams.error);

    let is_background = tokens.arguments.last().map(|s| s.as_str()) == Some("&");
    let args = if is_background {
        &tokens.arguments[..tokens.arguments.len() - 1]
    } else {
        &tokens.arguments
    };

    let is_background_redirect_stdout =
        is_background && matches!(io_streams.output, OutputStream::Stdout);
    let is_background_redirect_stderr =
        is_background && matches!(io_streams.error, OutputStream::Stderr);

    let stdout_cfg = if is_background_redirect_stdout {
        Stdio::piped()
    } else {
        io_streams.output.into_stdio()
    };

    let stderr_cfg = if is_background_redirect_stderr {
        Stdio::piped()
    } else {
        io_streams.error.into_stdio()
    };

    let mut command_binding = Command::new(&tokens.command);
    let command = command_binding
        .stdin(io_streams.input.into_stdio())
        .stdout(stdout_cfg)
        .stderr(stderr_cfg);

    match command.args(args).spawn() {
        Ok(mut child) => {
            if is_background {
                let shell_state_clone = Arc::clone(&shell_state);
                let pid = child.id();

                let job_id = {
                    let mut guard = shell_state_clone.write().unwrap();
                    let id = (1..)
                        .find(|&smallest| {
                            !guard.background_jobs.iter().any(|job| job.id() == smallest)
                        })
                        .unwrap();

                    guard.background_jobs.push(BackgroundJob::new(
                        id,
                        pid,
                        format!("{} {}", tokens.command, tokens.arguments.join(" ")),
                        BackgroundJobStatus::Running,
                    ));
                    id
                };

                writeln!(OutputStream::Stdout, "[{}] {}", job_id, pid).ok();

                if is_background_redirect_stdout && let Some(stdout) = child.stdout.take() {
                    spawn_reader(BufReader::new(stdout), printer.clone());
                }

                if is_background_redirect_stderr && let Some(stderr) = child.stderr.take() {
                    spawn_reader(BufReader::new(stderr), printer.clone());
                }

                std::thread::spawn(move || {
                    child.wait().unwrap();

                    let mut guard = shell_state_clone.write().unwrap();
                    let len = guard.background_jobs.len();

                    if let Some((idx, job)) = guard
                        .background_jobs
                        .iter_mut()
                        .enumerate()
                        .find(|(_, job)| job.id() == job_id)
                    {
                        job.set_status(BackgroundJobStatus::Done);
                        job.strip_command_suffix();

                        printer.print(job.format_job_output(idx, len)).unwrap();
                    }
                });

                None
            } else {
                Some(child)
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            writeln!(fallback_error, "{}: command not found", tokens.command).ok();
            None
        }
        Err(e) => {
            writeln!(
                fallback_error,
                "{}: error executing command: {}",
                tokens.command, e
            )
            .ok();
            None
        }
    }
}

fn spawn_reader<R: Read + Send + 'static>(reader_stream: R, printer: ExternalPrinter<String>) {
    std::thread::spawn(move || {
        let reader = BufReader::new(reader_stream);
        for line in reader.lines().map_while(Result::ok) {
            printer.print(line).ok();
        }
    });
}
