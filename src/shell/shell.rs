use crate::command::Command;
use crate::shell::shell_helper::ShellHelper;
use crate::system::env::get_env_path_executables;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Editor};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};
use std::{process, thread};

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

pub struct ShellState {
    pub completion_specifications: HashMap<String, String>,
    pub background_jobs: Vec<BackgroundJob>,
}

pub struct Shell;

impl Shell {
    pub fn start_session() {
        let shell_state = Arc::new(RwLock::new(ShellState {
            completion_specifications: HashMap::new(),
            background_jobs: vec![],
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

                    Command::run_command(Command::parse_command(&input), Arc::clone(&shell_state));
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
