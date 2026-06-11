use crate::shell::history::History;
use crate::shell::jobs::BackgroundJob;
use std::collections::HashMap;

pub struct ShellState {
    pub completion_specifications: HashMap<String, String>,
    pub background_jobs: Vec<BackgroundJob>,
    pub history: History,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            completion_specifications: HashMap::new(),
            background_jobs: vec![],
            history: History {
                entries: History::startup_history_file(),
                append_index: 0,
            },
        }
    }
}
