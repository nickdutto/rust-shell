use crate::shell::background_jobs::BackgroundJobs;
use crate::shell::history::History;
use crate::shell::variables::Variables;
use std::collections::HashMap;

pub struct ShellState {
    pub background_jobs: BackgroundJobs,
    pub completion_specifications: HashMap<String, String>,
    pub history: History,
    pub variables: Variables,
}

impl Default for ShellState {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            background_jobs: BackgroundJobs::new(),
            completion_specifications: HashMap::new(),
            history: History::default(),
            variables: Variables::new(),
        }
    }
}
