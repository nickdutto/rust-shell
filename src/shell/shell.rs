use crate::command::job::Job;
use crate::shell::background_jobs::BackgroundJobStatus;
use crate::shell::shell_helper::ShellHelper;
use crate::shell::shell_state::ShellState;
use crate::system::env::get_env_path_executables;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, Editor};
use std::sync::{Arc, RwLock};
use std::{process, thread};

pub struct Shell {
    editor: Editor<ShellHelper, DefaultHistory>,
    shell_state: Arc<RwLock<ShellState>>,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    pub fn new() -> Self {
        Self {
            editor: Editor::new().unwrap(),
            shell_state: Arc::new(RwLock::new(ShellState::new())),
        }
    }

    pub fn start_session(&mut self) {
        let shell_helper =
            ShellHelper::new(Shell::get_path_executables(), Arc::clone(&self.shell_state));
        self.editor.set_helper(Some(shell_helper));
        self.editor.set_completion_type(CompletionType::List);

        self.repl();
    }

    fn repl(&mut self) {
        loop {
            match self.editor.readline("$ ") {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    self.editor.add_history_entry(line.as_str()).ok();
                    self.shell_state
                        .write()
                        .unwrap()
                        .history
                        .entries
                        .push(line.as_str().to_string());

                    if let Some(job) =
                        { Job::parse_line(&line, &self.shell_state.read().unwrap().variables) }
                    {
                        job.run(Arc::clone(&self.shell_state));
                    }

                    Shell::print_background_jobs(Arc::clone(&self.shell_state));
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

    fn get_path_executables() -> Arc<RwLock<Vec<String>>> {
        let path_executables = Arc::new(RwLock::new(Vec::new()));
        let path_executables_bg = Arc::clone(&path_executables);

        thread::spawn(move || {
            let executables = get_env_path_executables("PATH");
            if let Ok(mut guard) = path_executables_bg.write() {
                *guard = executables;
            }
        });

        path_executables
    }

    fn print_background_jobs(shell_state: Arc<RwLock<ShellState>>) {
        let jobs_list = shell_state
            .read()
            .unwrap()
            .background_jobs
            .to_list_string(Some(BackgroundJobStatus::Done));

        if !jobs_list.is_empty() {
            println!("{jobs_list}");
        }

        shell_state
            .write()
            .unwrap()
            .background_jobs
            .remove_done_jobs();
    }
}
