use crate::command::Command;
use crate::command::builtin::jobs::format_job_output;
use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::shell::shell_helper::ShellHelper;
use crate::system::env::get_env_path_executables;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Editor};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};
use std::{process, thread};

#[derive(PartialEq)]
pub enum BackgroundJobStatus {
    Done,
    Running,
}

impl Display for BackgroundJobStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BackgroundJobStatus::Done => f.pad("Done"),
            BackgroundJobStatus::Running => f.pad("Running"),
        }
    }
}

pub struct BackgroundJob {
    pub id: usize,
    pub pid: u32,
    pub command: String,
    pub status: BackgroundJobStatus,
}

pub struct History {
    pub entries: Vec<String>,
    pub append_index: usize,
}

pub struct ShellState {
    pub completion_specifications: HashMap<String, String>,
    pub background_jobs: Vec<BackgroundJob>,
    pub history: History,
}

pub struct Shell;

impl Shell {
    pub fn start_session() {
        let shell_state = Arc::new(RwLock::new(ShellState {
            completion_specifications: HashMap::new(),
            background_jobs: vec![],
            history: History {
                entries: vec![],
                append_index: 0,
            },
        }));

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

                    Command::run_command(Command::parse_command(&input), Arc::clone(&shell_state));

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
                        &Tokens {
                            command: String::new(),
                            arguments: vec![],
                            redirection: None,
                        },
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
