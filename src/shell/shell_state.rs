use crate::shell::background_jobs::BackgroundJobs;
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
}
