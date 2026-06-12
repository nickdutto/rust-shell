use crate::command::Command;
use crate::command::builtin::jobs::format_job_output;
use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::shell::jobs::BackgroundJobStatus;
use crate::shell::shell_helper::ShellHelper;
use crate::shell::shell_state::ShellState;
use crate::system::env::get_env_path_executables;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Editor};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::{process, thread};

pub struct Shell;

impl Shell {
    pub fn start_session() {
        let shell_state = Arc::new(RwLock::new(ShellState::new()));

        let executable_completions = Arc::new(RwLock::new(Vec::new()));
        let executable_completions_bg = Arc::clone(&executable_completions);

        thread::spawn(move || {
            let executables = get_env_path_executables("PATH");
            if let Ok(mut guard) = executable_completions_bg.write() {
                *guard = executables;
            }
        });

        let shell_state_for_helper = Arc::clone(&shell_state);
        let mut rl = Editor::new().unwrap();
        rl.set_helper(Some(ShellHelper::new(
            executable_completions,
            shell_state_for_helper,
        )));
        rl.set_completion_type(CompletionType::List);

        loop {
            let readline = rl.readline("$ ");
            match readline {
                Ok(input) => {
                    if input.trim().is_empty() {
                        continue;
                    }

                    rl.add_history_entry(input.as_str()).ok();
                    {
                        let mut guard = shell_state.write().unwrap();
                        guard.history.entries.push(input.as_str().to_string())
                    }

                    let command =
                        { Command::parse_command(&input, &shell_state.read().unwrap().variables) };

                    Command::run_command(command, Arc::clone(&shell_state));

                    let mut job_pids_to_remove: HashSet<u32> = HashSet::new();
                    let jobs_output: Vec<String>;

                    {
                        let guard = shell_state.read().unwrap();
                        jobs_output = guard
                            .background_jobs
                            .iter()
                            .enumerate()
                            .filter(|(_, job)| job.status == BackgroundJobStatus::Done)
                            .map(|(idx, job)| {
                                job_pids_to_remove.insert(job.pid);
                                format_job_output(job, idx, guard.background_jobs.len())
                            })
                            .collect();
                    }

                    write_output(
                        jobs_output.join("\n").to_string().trim(),
                        OutputType::Stdout,
                        Some(&Tokens {
                            command: String::new(),
                            arguments: vec![],
                            redirection: None,
                        }),
                    );

                    {
                        let mut guard = shell_state.write().unwrap();
                        guard
                            .background_jobs
                            .retain(|job| !job_pids_to_remove.contains(&job.pid));
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    process::exit(0);
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
}
