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

pub struct Shell<'a> {
    editor: Editor<ShellHelper<'a>, DefaultHistory>,
    shell_state: Arc<RwLock<ShellState>>,
}

impl<'a> Default for Shell<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Shell<'a> {
    pub fn new() -> Self {
        Self {
            editor: Editor::new().unwrap(),
            shell_state: Arc::new(RwLock::new(ShellState::new())),
        }
    }

    pub fn start_session(&mut self) {
        self.editor.set_completion_type(CompletionType::List);
        self.editor.set_helper(Some(ShellHelper::new(
            Shell::build_executable_completions_in_background(),
            Arc::clone(&self.shell_state),
        )));

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
                    {
                        let mut guard = self.shell_state.write().unwrap();
                        guard.history.entries.push(line.as_str().to_string())
                    }

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

    fn build_executable_completions_in_background() -> Arc<RwLock<Vec<String>>> {
        let executable_completions = Arc::new(RwLock::new(Vec::new()));
        let executable_completions_bg = Arc::clone(&executable_completions);

        thread::spawn(move || {
            let executables = get_env_path_executables("PATH");
            if let Ok(mut guard) = executable_completions_bg.write() {
                *guard = executables;
            }
        });

        executable_completions
    }

    fn print_background_jobs(shell_state: Arc<RwLock<ShellState>>) {
        {
            let guard = shell_state.read().unwrap();
            let output: Vec<String> = guard
                .background_jobs
                .iter()
                .enumerate()
                .filter(|(_, job)| job.status() == BackgroundJobStatus::Done)
                .map(|(idx, job)| job.format_job_output(idx, guard.background_jobs.len()))
                .collect();

            if !output.is_empty() {
                println!("{}", output.join("\n").to_string().trim());
            }
        }

        shell_state
            .write()
            .unwrap()
            .background_jobs
            .remove_done_jobs();
    }
}
