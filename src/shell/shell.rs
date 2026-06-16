use crate::command::job::Job;
use crate::shell::background_jobs::BackgroundJobStatus;
use crate::shell::shell_helper::ShellHelper;
use crate::shell::shell_state::ShellState;
use crate::system::env::get_env_path_executables;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Editor};
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

                    if let Some(job) =
                        { Job::parse_line(&input, &shell_state.read().unwrap().variables) }
                    {
                        job.run(Arc::clone(&shell_state));
                    }

                    let jobs_output: Vec<String>;

                    {
                        let guard = shell_state.read().unwrap();
                        jobs_output = guard
                            .background_jobs
                            .iter()
                            .enumerate()
                            .filter(|(_, job)| job.status() == BackgroundJobStatus::Done)
                            .map(|(idx, job)| {
                                job.format_job_output(idx, guard.background_jobs.len())
                            })
                            .collect();
                    }

                    if !jobs_output.is_empty() {
                        println!("{}", jobs_output.join("\n").to_string().trim());
                    }

                    {
                        let mut guard = shell_state.write().unwrap();
                        guard.background_jobs.remove_done_jobs();
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    process::exit(0);
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    break;
                }
            }
        }
    }
}
