use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::shell::{BackgroundJob, BackgroundJobStatus, ShellState};
use std::io::{BufReader, ErrorKind, Read};
use std::process;
use std::process::{Child, Stdio};
use std::sync::{Arc, RwLock};

pub fn handle_executable(tokens: Tokens, shell_state: Arc<RwLock<ShellState>>) {
    if tokens.command.is_empty() {
        return;
    }

    let mut command_binding = process::Command::new(&tokens.command);
    let command = command_binding
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if tokens
        .arguments
        .iter()
        .next_back()
        .unwrap_or(&"".to_string())
        == "&"
    {
        match_spawn_process(
            command
                .args(&tokens.arguments[..tokens.arguments.len() - 1])
                .spawn(),
            true,
            tokens,
            shell_state,
        );
    } else {
        match_spawn_process(
            command.args(&tokens.arguments).spawn(),
            false,
            tokens,
            shell_state,
        )
    }

    fn match_spawn_process(
        result: std::io::Result<Child>,
        run_background_job: bool,
        tokens: Tokens,
        shell_state: Arc<RwLock<ShellState>>,
    ) {
        let mut stdout_output = String::new();
        let mut stderr_output = String::new();

        match result {
            Ok(mut child) => {
                if run_background_job {
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

                    write_output(
                        format!("[{}] {}", job_id, pid).trim(),
                        OutputType::Stdout,
                        &tokens,
                    );

                    std::thread::spawn(move || {
                        if let Some(stdout) = child.stdout.take() {
                            let mut reader = BufReader::new(stdout);
                            reader.read_to_string(&mut stdout_output).unwrap();
                        }

                        if let Some(stderr) = child.stderr.take() {
                            let mut reader = BufReader::new(stderr);
                            reader.read_to_string(&mut stderr_output).unwrap();
                        }

                        child.wait().unwrap();

                        {
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
                        }

                        write_output(stdout_output.trim(), OutputType::Stdout, &tokens);
                        write_output(stderr_output.trim(), OutputType::Stderr, &tokens);
                    });
                } else {
                    if let Some(stdout) = child.stdout.take() {
                        let mut reader = BufReader::new(stdout);
                        reader.read_to_string(&mut stdout_output).unwrap();
                    }

                    if let Some(stderr) = child.stderr.take() {
                        let mut reader = BufReader::new(stderr);
                        reader.read_to_string(&mut stderr_output).unwrap();
                    }

                    child.wait().unwrap();

                    write_output(stdout_output.trim(), OutputType::Stdout, &tokens);
                    write_output(stderr_output.trim(), OutputType::Stderr, &tokens);
                }
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                write_output(
                    format!("{}: command not found", tokens.command).trim(),
                    OutputType::Stderr,
                    &tokens,
                );
            }
            Err(e) => {
                write_output(
                    format!("{}: error executing command: {}", tokens.command, e).trim(),
                    OutputType::Stderr,
                    &tokens,
                );
            }
        }
    }
}
