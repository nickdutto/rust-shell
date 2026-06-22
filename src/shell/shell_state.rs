use crate::shell::background_jobs::{BackgroundJobStatus, BackgroundJobs};
use crate::shell::completions::Completions;
use crate::shell::history::History;
use crate::shell::variables::Variables;

#[derive(Default)]
pub struct ShellState {
    pub background_jobs: BackgroundJobs,
    pub completions: Completions,
    pub history: History,
    pub variables: Variables,
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            background_jobs: BackgroundJobs::new(),
            completions: Completions::new(),
            history: History::new(),
            variables: Variables::new(),
        }
    }

    pub fn print_background_jobs(&mut self) {
        let jobs_list = self
            .background_jobs
            .to_list_string(Some(BackgroundJobStatus::Done));

        if !jobs_list.is_empty() {
            println!("{jobs_list}");
        }

        self.background_jobs.remove_done_jobs();
    }
}
