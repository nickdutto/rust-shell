use crate::shell::history::History;
use crate::shell::jobs::BackgroundJob;
use crate::shell::variables::Variables;
use std::collections::HashMap;

pub struct ShellState {
    pub background_jobs: Vec<BackgroundJob>,
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
            background_jobs: vec![],
            completion_specifications: HashMap::new(),
            history: History::default(),
            variables: Variables::new(),
        }
    }
}
